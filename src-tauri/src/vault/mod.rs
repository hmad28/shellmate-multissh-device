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

/// In-memory vault state. Holds the derived key only while unlocked.
pub struct Vault {
    inner: RwLock<VaultInner>,
}

struct VaultInner {
    key: Option<[u8; 32]>,
    last_activity: Instant,
    autolock: Duration,
}

impl Vault {
    pub fn new() -> Self {
        Self {
            inner: RwLock::new(VaultInner {
                key: None,
                last_activity: Instant::now(),
                autolock: Duration::from_secs(DEFAULT_AUTOLOCK_SECS),
            }),
        }
    }

    pub fn is_unlocked(&self) -> bool {
        self.inner.read().key.is_some()
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
    }

    pub fn set_autolock(&self, secs: u64) {
        let mut inner = self.inner.write();
        inner.autolock = Duration::from_secs(secs);
    }

    /// Returns true if a vault has already been initialized in the given DB.
    pub fn is_initialized(conn: &Connection) -> AppResult<bool> {
        let value = get_setting(conn, SETTING_VAULT_INITIALIZED)?;
        Ok(value.as_deref() == Some("1"))
    }

    /// Initialize the vault for the first time. Stores the salt and a verifier
    /// ciphertext. The derived key is held in memory (vault becomes unlocked).
    pub fn setup(&self, conn: &Connection, password: &str) -> AppResult<()> {
        if Self::is_initialized(conn)? {
            return Err(AppError::InvalidInput("vault already initialized".into()));
        }
        validate_password(password)?;

        let salt = crypto::generate_salt();
        let mut key = crypto::derive_key(password.as_bytes(), &salt)?;
        let verifier = crypto::encrypt(&key, VERIFIER_PLAINTEXT)?;

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
        inner.key = Some(key);
        inner.last_activity = Instant::now();
        // `key` is now owned by inner — local binding shadowed; zeroize to be safe.
        key = [0u8; 32];
        let _ = key;
        Ok(())
    }

    /// Try to unlock the vault using `password`. Returns Ok(()) on success.
    pub fn unlock(&self, conn: &Connection, password: &str) -> AppResult<()> {
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

        let mut key = crypto::derive_key(password.as_bytes(), &salt)?;
        let blob = EncryptedBlob { ciphertext, nonce };
        let plaintext = match crypto::decrypt(&key, &blob) {
            Ok(p) => p,
            Err(_) => {
                key.zeroize();
                return Err(AppError::InvalidInput("incorrect master password".into()));
            }
        };

        // Constant-time compare to defeat any plaintext-shape side channel.
        if plaintext.ct_eq(VERIFIER_PLAINTEXT).unwrap_u8() != 1 {
            key.zeroize();
            return Err(AppError::InvalidInput("incorrect master password".into()));
        }

        let mut inner = self.inner.write();
        inner.key = Some(key);
        inner.last_activity = Instant::now();
        // local key already moved into inner; reset binding for safety
        key = [0u8; 32];
        let _ = key;

        // Refresh autolock from settings (in case user changed it).
        if let Ok(Some(s)) = get_setting(conn, SETTING_VAULT_AUTOLOCK_SECS) {
            if let Ok(secs) = s.parse::<u64>() {
                inner.autolock = Duration::from_secs(secs);
            }
        }
        Ok(())
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
