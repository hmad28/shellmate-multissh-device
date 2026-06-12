use crate::errors::{AppError, AppResult};
use crate::vault::Vault;
use ed25519_dalek::{SigningKey, VerifyingKey};
use parking_lot::Mutex;
use rusqlite::Connection;
use std::path::PathBuf;
use std::sync::Arc;
use tauri::State;

pub struct VipKeyStore {
    inner: Mutex<Option<VipKeyPair>>,
}

pub struct VipKeyPair {
    pub signing_key: SigningKey,
    pub verifying_key: VerifyingKey,
}

impl VipKeyStore {
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(None),
        }
    }

    pub fn set(&self, keypair: VipKeyPair) {
        *self.inner.lock() = Some(keypair);
    }

    pub fn get(&self) -> Option<VipKeyPair> {
        self.inner.lock().clone()
    }

    pub fn clear(&self) {
        *self.inner.lock() = None;
    }
}

impl Clone for VipKeyPair {
    fn clone(&self) -> Self {
        Self {
            signing_key: self.signing_key.clone(),
            verifying_key: self.verifying_key,
        }
    }
}

fn get_authorized_keys_path() -> AppResult<PathBuf> {
    if cfg!(target_os = "windows") {
        let home = dirs::home_dir().ok_or_else(|| AppError::Internal("cannot determine home directory".into()))?;
        Ok(home.join(".ssh").join("authorized_keys"))
    } else if cfg!(target_os = "macos") || cfg!(target_os = "linux") {
        let home = dirs::home_dir().ok_or_else(|| AppError::Internal("cannot determine home directory".into()))?;
        Ok(home.join(".ssh").join("authorized_keys"))
    } else {
        Err(AppError::Internal("VIP access not supported on this platform".into()))
    }
}

#[tauri::command]
pub async fn vip_generate_keypair(
    state: State<'_, Arc<VipKeyStore>>,
    db: State<'_, Arc<Mutex<Connection>>>,
    vault: State<'_, Arc<Vault>>,
) -> AppResult<String> {
    let mut csprng = rand_core::OsRng;
    let signing_key = SigningKey::generate(&mut csprng);
    let verifying_key = signing_key.verifying_key();

    let pubkey_hex = hex::encode(verifying_key.as_bytes());
    let privkey_bytes = signing_key.to_bytes();

    // Encrypt private key with vault and store as credential
    let credential_id = {
        let conn = db.lock();
        let encrypted = vault.encrypt(&privkey_bytes)?;
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        conn.execute(
            "INSERT INTO credentials (id, type, encrypted_data, nonce, created_at, updated_at)
             VALUES (?1, 'private_key', ?2, ?3, ?4, ?5)",
            rusqlite::params![id, encrypted.ciphertext, encrypted.nonce.to_vec(), now, now],
        )?;
        id
    };

    state.set(VipKeyPair {
        signing_key,
        verifying_key,
    });

    Ok(serde_json::json!({
        "publicKey": pubkey_hex,
        "credentialId": credential_id,
    })
    .to_string())
}

#[tauri::command]
pub async fn vip_inject_authorized_keys(
    pubkey_hex: String,
) -> AppResult<String> {
    let key_path = get_authorized_keys_path()?;

    // Ensure .ssh directory exists
    if let Some(parent) = key_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let public_key_line = format!("ssh-ed25519 {} shellmate-vip", pubkey_hex);

    // Read existing authorized_keys
    let existing = std::fs::read_to_string(&key_path).unwrap_or_default();

    // Check if key already exists
    if existing.contains(&public_key_line) {
        return Ok("Key already exists in authorized_keys".to_string());
    }

    // Append the key
    let mut content = existing.trim_end().to_string();
    if !content.is_empty() {
        content.push('\n');
    }
    content.push_str(&public_key_line);
    content.push('\n');

    std::fs::write(&key_path, content)?;

    Ok(format!("Injected public key into {}", key_path.display()))
}

#[tauri::command]
pub async fn vip_create_localhost_host(
    db: State<'_, Arc<Mutex<Connection>>>,
    credential_id: String,
    label: Option<String>,
) -> AppResult<String> {
    let conn = db.lock();

    // Check if a localhost VIP host already exists
    let existing: Option<String> = conn
        .query_row(
            "SELECT id FROM hosts WHERE hostname = 'localhost' AND label LIKE 'VIP%' LIMIT 1",
            [],
            |row| row.get(0),
        )
        .ok();

    if let Some(id) = existing {
        return Ok(id);
    }

    let host_id = uuid::Uuid::new_v4().to_string();
    let host_label = label.unwrap_or_else(|| "VIP Localhost".to_string());
    let now = chrono::Utc::now().to_rfc3339();

    conn.execute(
        "INSERT INTO hosts (id, label, hostname, port, username, auth_type, credential_id, tags, created_at, updated_at)
         VALUES (?1, ?2, 'localhost', 22, 'Administrator', 'key', ?3, '[\"vip\"]', ?4, ?5)",
        rusqlite::params![host_id, host_label, credential_id, now, now],
    )?;

    Ok(host_id)
}

#[tauri::command]
pub async fn vip_get_key_status(
    db: State<'_, Arc<Mutex<Connection>>>,
) -> AppResult<serde_json::Value> {
    let conn = db.lock();

    // Check if VIP host exists
    let host_exists: bool = conn
        .query_row(
            "SELECT COUNT(*) FROM hosts WHERE hostname = 'localhost' AND label LIKE 'VIP%'",
            [],
            |row| row.get::<_, i64>(0),
        )
        .map(|c| c > 0)
        .unwrap_or(false);

    // Check authorized_keys
    let auth_keys_injected = get_authorized_keys_path()
        .map(|path| {
            std::fs::read_to_string(&path)
                .map(|content| content.contains("shellmate-vip"))
                .unwrap_or(false)
        })
        .unwrap_or(false);

    Ok(serde_json::json!({
        "hostExists": host_exists,
        "authorizedKeysInjected": auth_keys_injected,
    }))
}
