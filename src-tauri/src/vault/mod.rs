use crate::crypto::{self, EncryptedBlob, NONCE_LEN, SALT_LEN};
use crate::errors::{AppError, AppResult};
use parking_lot::RwLock;
use rusqlite::Connection;
use std::time::{Duration, Instant};
use subtle::ConstantTimeEq;
use zeroize::Zeroize;

/// Settings keys used by the vault layer.
const SETTING_VAULT_INITIALIZED: &str = "vault.initialized";
const SETTING_VAULT_SALT: &str = "vault.salt";
const SETTING_VAULT_VERIFIER_CIPHERTEXT: &str = "vault.verifier.ciphertext";
const SETTING_VAULT_VERIFIER_NONCE: &str = "vault.verifier.nonce";
const SETTING_VAULT_AUTOLOCK_SECS: &str = "vault.autolock_secs";

/// Plaintext written into the verifier blob. Decryption recovering this exact
/// value confirms the master password is correct.
const VERIFIER_PLAINTEXT: &[u8] = b"shellmate.vault.v1";

const DEFAULT_AUTOLOCK_SECS: u64 = 15 * 60;

/// In-memory vault state. Holds the derived keys only while unlocked.
pub struct Vault {
    inner: RwLock<VaultInner>,
}

struct VaultInner {
    key: Option<[u8; 32]>,
    db_key: Option<[u8; 32]>,
    last_activity: Instant,
    autolock: Duration,
}

impl Vault {
    pub fn new() -> Self {
        Self {
            inner: RwLock::new(VaultInner {
                key: None,
                db_key: None,
                last_activity: Instant::now(),
                autolock: Duration::from_secs(DEFAULT_AUTOLOCK_SECS),
            }),
        }
    }

    pub fn is_unlocked(&self) -> bool {
        self.inner.read().key.is_some()
    }

    /// Returns the DB key if the vault is unlocked, None otherwise.
    pub fn get_db_key(&self) -> Option<[u8; 32]> {
        self.inner.read().db_key
    }

    /// Returns the vault key if the vault is unlocked, None otherwise.
    pub fn get_vault_key(&self) -> Option<[u8; 32]> {
        self.inner.read().key
    }

    pub fn record_activity(&self) {
        let mut inner = self.inner.write();
        inner.last_activity = Instant::now();
    }

    /// Returns true if the vault was auto-locked due to idle timeout.
    pub fn check_idle(&self) -> bool {
        let mut inner = self.inner.write();
        if inner.key.is_some()
            && !inner.autolock.is_zero()
            && inner.last_activity.elapsed() >= inner.autolock
        {
            if let Some(mut key) = inner.key.take() {
                key.zeroize();
            }
            if let Some(mut db_key) = inner.db_key.take() {
                db_key.zeroize();
            }
            true
        } else {
            false
        }
    }

    pub fn lock(&self) {
        let mut inner = self.inner.write();
        if let Some(mut key) = inner.key.take() {
            key.zeroize();
        }
        if let Some(mut db_key) = inner.db_key.take() {
            db_key.zeroize();
        }
    }

    pub fn set_autolock(&self, secs: u64) {
        let mut inner = self.inner.write();
        inner.autolock = Duration::from_secs(secs);
    }

    /// Unlock the vault directly with a derived key (used by biometric unlock).
    /// The key must be the vault key (not the master key). The db_key is derived
    /// from the same master key via HKDF.
    pub fn unlock_with_key(&self, master_key: &[u8; 32]) -> AppResult<()> {
        let (vault_key, db_key) = crypto::derive_vault_and_db_keys(master_key);
        let mut inner = self.inner.write();
        if let Some(mut prev) = inner.key.replace(vault_key) {
            prev.zeroize();
        }
        if let Some(mut prev) = inner.db_key.replace(db_key) {
            prev.zeroize();
        }
        inner.last_activity = Instant::now();
        Ok(())
    }

    /// Returns true if a vault has already been initialized in the given DB.
    pub fn is_initialized(conn: &Connection) -> AppResult<bool> {
        let value = get_setting(conn, SETTING_VAULT_INITIALIZED)?;
        Ok(value.as_deref() == Some("1"))
    }

    /// Check if vault metadata file exists (DB is encrypted).
    pub fn has_meta(db_path: &std::path::Path) -> bool {
        crate::db::has_vault_meta(db_path)
    }

    /// Verify password using vault metadata file (no DB needed).
    /// Returns the master key on success.
    pub fn verify_from_meta(
        db_path: &std::path::Path,
        password: &str,
    ) -> AppResult<[u8; 32]> {
        let (salt, nonce_bytes, ciphertext) = crate::db::read_vault_meta(db_path)?;

        let mut master_key = crypto::derive_key(password.as_bytes(), &salt)?;
        let (vault_key, _db_key) = crypto::derive_vault_and_db_keys(&master_key);

        let nonce = unfixed_nonce(&nonce_bytes)?;
        let blob = EncryptedBlob { ciphertext, nonce };
        let plaintext = match crypto::decrypt(&vault_key, &blob) {
            Ok(p) => p,
            Err(_) => {
                master_key.zeroize();
                return Err(AppError::InvalidInput("incorrect master password".into()));
            }
        };

        if plaintext.ct_eq(VERIFIER_PLAINTEXT).unwrap_u8() != 1 {
            master_key.zeroize();
            return Err(AppError::InvalidInput("incorrect master password".into()));
        }

        // Don't zeroize master_key — caller needs it to derive vault/db keys.
        Ok(master_key)
    }

    /// Initialize the vault for the first time. Stores the salt and a verifier
    /// ciphertext. The derived key is held in memory (vault becomes unlocked).
    /// Returns the DB key for SQLCipher encryption.
    pub fn setup(&self, conn: &Connection, password: &str) -> AppResult<[u8; 32]> {
        if Self::is_initialized(conn)? {
            return Err(AppError::InvalidInput("vault already initialized".into()));
        }
        validate_password(password)?;

        let salt = crypto::generate_salt();
        let mut master_key = crypto::derive_key(password.as_bytes(), &salt)?;
        let (vault_key, db_key) = crypto::derive_vault_and_db_keys(&master_key);
        master_key.zeroize();
        let verifier = crypto::encrypt(&vault_key, VERIFIER_PLAINTEXT)?;

        set_setting(conn, SETTING_VAULT_SALT, &hex(&salt))?;
        set_setting(
            conn,
            SETTING_VAULT_VERIFIER_CIPHERTEXT,
            &hex(&verifier.ciphertext),
        )?;
        set_setting(
            conn,
            SETTING_VAULT_VERIFIER_NONCE,
            &hex(&verifier.nonce),
        )?;
        set_setting(conn, SETTING_VAULT_AUTOLOCK_SECS, &DEFAULT_AUTOLOCK_SECS.to_string())?;
        set_setting(conn, SETTING_VAULT_INITIALIZED, "1")?;

        let mut inner = self.inner.write();
        inner.key = Some(vault_key);
        inner.db_key = Some(db_key);
        inner.last_activity = Instant::now();
        Ok(db_key)
    }

    /// Try to unlock the vault using `password`. Returns the DB key on success.
    pub fn unlock(&self, conn: &Connection, password: &str) -> AppResult<[u8; 32]> {
        if !Self::is_initialized(conn)? {
            return Err(AppError::InvalidInput("vault not initialized".into()));
        }

        let salt_hex = get_setting(conn, SETTING_VAULT_SALT)?
            .ok_or_else(|| AppError::Internal("vault salt missing".into()))?;
        let ct_hex = get_setting(conn, SETTING_VAULT_VERIFIER_CIPHERTEXT)?
            .ok_or_else(|| AppError::Internal("vault verifier ciphertext missing".into()))?;
        let nonce_hex = get_setting(conn, SETTING_VAULT_VERIFIER_NONCE)?
            .ok_or_else(|| AppError::Internal("vault verifier nonce missing".into()))?;

        let salt = unhex_fixed::<SALT_LEN>(&salt_hex)?;
        let nonce = unhex_fixed::<NONCE_LEN>(&nonce_hex)?;
        let ciphertext = unhex(&ct_hex)?;

        let mut master_key = crypto::derive_key(password.as_bytes(), &salt)?;
        let (mut vault_key, mut db_key) = crypto::derive_vault_and_db_keys(&master_key);
        // Zeroize master key immediately — only vault_key and db_key are needed.
        master_key.zeroize();

        let blob = EncryptedBlob { ciphertext, nonce };
        let plaintext = match crypto::decrypt(&vault_key, &blob) {
            Ok(p) => p,
            Err(_) => {
                vault_key.zeroize();
                db_key.zeroize();
                return Err(AppError::InvalidInput("incorrect master password".into()));
            }
        };

        // Constant-time compare to defeat any plaintext-shape side channel.
        if plaintext.ct_eq(VERIFIER_PLAINTEXT).unwrap_u8() != 1 {
            vault_key.zeroize();
            db_key.zeroize();
            return Err(AppError::InvalidInput("incorrect master password".into()));
        }

        let mut inner = self.inner.write();
        inner.key = Some(vault_key);
        inner.db_key = Some(db_key);
        inner.last_activity = Instant::now();

        // Refresh autolock from settings (in case user changed it).
        if let Ok(Some(s)) = get_setting(conn, SETTING_VAULT_AUTOLOCK_SECS) {
            if let Ok(secs) = s.parse::<u64>() {
                inner.autolock = Duration::from_secs(secs);
            }
        }
        Ok(db_key)
    }

    /// Encrypt `plaintext` using the unlocked vault key.
    pub fn encrypt(&self, plaintext: &[u8]) -> AppResult<EncryptedBlob> {
        let inner = self.inner.read();
        let key = inner
            .key
            .ok_or_else(|| AppError::InvalidInput("vault is locked".into()))?;
        crypto::encrypt(&key, plaintext)
    }

    /// Decrypt `blob` using the unlocked vault key.
    pub fn decrypt(&self, blob: &EncryptedBlob) -> AppResult<Vec<u8>> {
        let inner = self.inner.read();
        let key = inner
            .key
            .ok_or_else(|| AppError::InvalidInput("vault is locked".into()))?;
        crypto::decrypt(&key, blob)
    }

    /// Change the master password. Re-derives the key, re-encrypts all stored
    /// credentials and the verifier blob, and atomically commits.
    /// Vault must be unlocked. The new password is validated against the policy.
    pub fn change_master_password(
        &self,
        conn: &mut Connection,
        current_password: &str,
        new_password: &str,
    ) -> AppResult<()> {
        if !self.is_unlocked() {
            return Err(AppError::InvalidInput("vault is locked".into()));
        }
        validate_password(new_password)?;

        // Verify current password by re-deriving + comparing with stored verifier.
        let salt_hex = get_setting(conn, SETTING_VAULT_SALT)?
            .ok_or_else(|| AppError::Internal("vault salt missing".into()))?;
        let ct_hex = get_setting(conn, SETTING_VAULT_VERIFIER_CIPHERTEXT)?
            .ok_or_else(|| AppError::Internal("vault verifier ciphertext missing".into()))?;
        let nonce_hex = get_setting(conn, SETTING_VAULT_VERIFIER_NONCE)?
            .ok_or_else(|| AppError::Internal("vault verifier nonce missing".into()))?;

        let current_salt = unhex_fixed::<SALT_LEN>(&salt_hex)?;
        let current_nonce = unhex_fixed::<NONCE_LEN>(&nonce_hex)?;
        let current_ciphertext = unhex(&ct_hex)?;

        let mut current_master_key = crypto::derive_key(current_password.as_bytes(), &current_salt)?;
        let (mut current_vault_key, _current_db_key) = crypto::derive_vault_and_db_keys(&current_master_key);
        current_master_key.zeroize();
        let blob = EncryptedBlob {
            ciphertext: current_ciphertext,
            nonce: current_nonce,
        };
        let plaintext = match crypto::decrypt(&current_vault_key, &blob) {
            Ok(p) => p,
            Err(_) => {
                current_vault_key.zeroize();
                return Err(AppError::InvalidInput("incorrect current password".into()));
            }
        };
        if plaintext.ct_eq(VERIFIER_PLAINTEXT).unwrap_u8() != 1 {
            current_vault_key.zeroize();
            return Err(AppError::InvalidInput("incorrect current password".into()));
        }

        // Derive new key with fresh salt.
        let new_salt = crypto::generate_salt();
        let mut new_master_key = crypto::derive_key(new_password.as_bytes(), &new_salt)?;
        let (mut new_vault_key, mut new_db_key) = crypto::derive_vault_and_db_keys(&new_master_key);
        new_master_key.zeroize();

        // Atomically:
        //   1. Re-encrypt every credential (encrypted_data + nonce columns)
        //   2. Replace verifier blob
        //   3. Replace stored salt
        let tx = conn.transaction()?;

        // Read all credentials, decrypt with current key, re-encrypt with new key.
        let credentials: Vec<(String, Vec<u8>, Vec<u8>)> = {
            let mut stmt = tx
                .prepare("SELECT id, encrypted_data, nonce FROM credentials")?;
            let rows = stmt.query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, Vec<u8>>(1)?,
                    row.get::<_, Vec<u8>>(2)?,
                ))
            })?;
            rows.collect::<Result<Vec<_>, _>>()?
        };

        let now = chrono::Utc::now().to_rfc3339();
        for (id, ct, nonce_bytes) in credentials {
            if nonce_bytes.len() != NONCE_LEN {
                current_vault_key.zeroize();
                new_vault_key.zeroize();
                new_db_key.zeroize();
                return Err(AppError::Internal(format!(
                    "credential {id} has invalid nonce length"
                )));
            }
            let mut nonce = [0u8; NONCE_LEN];
            nonce.copy_from_slice(&nonce_bytes);
            let blob = EncryptedBlob { ciphertext: ct, nonce };
            let plaintext = match crypto::decrypt(&current_vault_key, &blob) {
                Ok(p) => p,
                Err(_) => {
                    current_vault_key.zeroize();
                    new_vault_key.zeroize();
                    new_db_key.zeroize();
                    return Err(AppError::Internal(format!(
                        "failed to decrypt credential {id} during rotation"
                    )));
                }
            };
            let new_blob = match crypto::encrypt(&new_vault_key, &plaintext) {
                Ok(b) => b,
                Err(e) => {
                    current_vault_key.zeroize();
                    new_vault_key.zeroize();
                    new_db_key.zeroize();
                    return Err(e);
                }
            };
            // zeroize plaintext copy
            let _ = plaintext;
            tx.execute(
                "UPDATE credentials SET encrypted_data = ?1, nonce = ?2, updated_at = ?3
                 WHERE id = ?4",
                rusqlite::params![new_blob.ciphertext, new_blob.nonce.to_vec(), now, id],
            )?;
        }

        // Re-encrypt verifier with new key
        let new_verifier = match crypto::encrypt(&new_vault_key, VERIFIER_PLAINTEXT) {
            Ok(b) => b,
            Err(e) => {
                current_vault_key.zeroize();
                new_vault_key.zeroize();
                new_db_key.zeroize();
                return Err(e);
            }
        };

        // Update settings (salt + verifier)
        tx.execute(
            "INSERT INTO settings (key, value) VALUES (?1, ?2)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            rusqlite::params![SETTING_VAULT_SALT, hex(&new_salt)],
        )?;
        tx.execute(
            "INSERT INTO settings (key, value) VALUES (?1, ?2)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            rusqlite::params![
                SETTING_VAULT_VERIFIER_CIPHERTEXT,
                hex(&new_verifier.ciphertext)
            ],
        )?;
        tx.execute(
            "INSERT INTO settings (key, value) VALUES (?1, ?2)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            rusqlite::params![
                SETTING_VAULT_VERIFIER_NONCE,
                hex(&new_verifier.nonce)
            ],
        )?;

        tx.commit()?;

        // Swap in-memory keys
        let mut inner = self.inner.write();
        if let Some(mut prev) = inner.key.replace(new_vault_key) {
            prev.zeroize();
        }
        if let Some(mut prev) = inner.db_key.replace(new_db_key) {
            prev.zeroize();
        }
        inner.last_activity = Instant::now();
        current_vault_key.zeroize();

        Ok(())
    }
}

impl Default for Vault {
    fn default() -> Self {
        Self::new()
    }
}

fn validate_password(password: &str) -> AppResult<()> {
    if password.chars().count() < 12 {
        return Err(AppError::InvalidInput(
            "master password must be at least 12 characters".into(),
        ));
    }
    if password.chars().count() > 128 {
        return Err(AppError::InvalidInput(
            "master password must be at most 128 characters".into(),
        ));
    }
    Ok(())
}

fn get_setting(conn: &Connection, key: &str) -> AppResult<Option<String>> {
    let result = conn
        .query_row(
            "SELECT value FROM settings WHERE key = ?1",
            [key],
            |row| row.get::<_, String>(0),
        )
        .map(Some)
        .or_else(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => Ok(None),
            other => Err(other),
        })?;
    Ok(result)
}

fn set_setting(conn: &Connection, key: &str, value: &str) -> AppResult<()> {
    conn.execute(
        "INSERT INTO settings (key, value) VALUES (?1, ?2)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        rusqlite::params![key, value],
    )?;
    Ok(())
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn unhex(s: &str) -> AppResult<Vec<u8>> {
    if s.len() % 2 != 0 {
        return Err(AppError::Internal("invalid hex length".into()));
    }
    (0..s.len())
        .step_by(2)
        .map(|i| {
            u8::from_str_radix(&s[i..i + 2], 16)
                .map_err(|e| AppError::Internal(format!("invalid hex: {e}")))
        })
        .collect()
}

fn unhex_fixed<const N: usize>(s: &str) -> AppResult<[u8; N]> {
    let v = unhex(s)?;
    if v.len() != N {
        return Err(AppError::Internal(format!(
            "expected {N} bytes, got {}",
            v.len()
        )));
    }
    let mut out = [0u8; N];
    out.copy_from_slice(&v);
    Ok(out)
}

fn unfixed_nonce(v: &[u8]) -> AppResult<[u8; NONCE_LEN]> {
    if v.len() != NONCE_LEN {
        return Err(AppError::Internal(format!(
            "expected {NONCE_LEN}-byte nonce, got {}",
            v.len()
        )));
    }
    let mut out = [0u8; NONCE_LEN];
    out.copy_from_slice(v);
    Ok(out)
}
