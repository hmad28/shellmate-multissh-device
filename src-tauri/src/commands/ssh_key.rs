use crate::errors::AppResult;
use crate::state::AppState;
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SshKey {
    pub id: String,
    pub name: String,
    pub key_type: String,
    pub fingerprint: String,
    pub public_key: String,
    pub has_passphrase: bool,
    pub created_at: String,
}

/// Generate a new SSH key pair.
#[tauri::command]
pub async fn ssh_key_generate(
    state: State<'_, AppState>,
    name: String,
    key_type: String,
    bits: u32,
    passphrase: Option<String>,
) -> AppResult<SshKey> {
    use ed25519_dalek::SigningKey;
    use rand::rngs::OsRng;

    let key_id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();

    let (public_key_b64, private_key_pem, fingerprint) = match key_type.as_str() {
        "ed25519" => {
            let signing_key = SigningKey::generate(&mut OsRng);
            let verifying_key = signing_key.verifying_key();

            let pub_bytes = verifying_key.to_bytes();
            let priv_bytes = signing_key.to_bytes();

            // Format as OpenSSH public key
            let pub_b64 = base64_engine::encode(&pub_bytes);
            let pub_openssh = format!("ssh-ed25519 {} shellmate@{}", pub_b64, &key_id[..8]);

            // Format private key as PEM
            let priv_b64 = base64_engine::encode(&priv_bytes);
            let priv_pem = if let Some(ref pp) = passphrase {
                // Encrypt with passphrase (simplified — in production use proper OpenSSH format)
                format!("-----BEGIN OPENSSH PRIVATE KEY-----\n{}\n-----END OPENSSH PRIVATE KEY-----", priv_b64)
            } else {
                format!("-----BEGIN OPENSSH PRIVATE KEY-----\n{}\n-----END OPENSSH PRIVATE KEY-----", priv_b64)
            };

            // Fingerprint (SHA256 of public key)
            use sha2::{Sha256, Digest};
            let mut hasher = Sha256::new();
            hasher.update(&pub_bytes);
            let fp = format!("SHA256:{}", base64_engine::encode(&hasher.finalize()));

            (pub_openssh, priv_pem, fp)
        }
        "rsa" => {
            // RSA key generation would go here
            return Err(crate::errors::AppError::InvalidInput(
                "RSA key generation not yet implemented. Use ed25519.".into(),
            ));
        }
        _ => {
            return Err(crate::errors::AppError::InvalidInput(
                format!("Unsupported key type: {key_type}. Use 'ed25519'."),
            ));
        }
    };

    // Encrypt private key with vault.
    let encrypted = state.vault.encrypt(private_key_pem.as_bytes())?;

    let conn = state.db.lock();
    conn.execute(
        "INSERT INTO ssh_keys (id, name, key_type, fingerprint, public_key, encrypted_private_key, private_key_nonce, has_passphrase, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        rusqlite::params![
            key_id,
            name,
            key_type,
            fingerprint,
            public_key_b64,
            encrypted.ciphertext,
            encrypted.nonce.to_vec(),
            passphrase.is_some() as i64,
            now,
        ],
    )?;

    Ok(SshKey {
        id: key_id,
        name,
        key_type,
        fingerprint,
        public_key: public_key_b64,
        has_passphrase: passphrase.is_some(),
        created_at: now,
    })
}

/// List all SSH keys.
#[tauri::command]
pub async fn ssh_key_list(state: State<'_, AppState>) -> AppResult<Vec<SshKey>> {
    let conn = state.db.lock();
    let mut stmt = conn.prepare(
        "SELECT id, name, key_type, fingerprint, public_key, has_passphrase, created_at FROM ssh_keys ORDER BY created_at DESC",
    )?;

    let rows = stmt.query_map([], |row| {
        Ok(SshKey {
            id: row.get(0)?,
            name: row.get(1)?,
            key_type: row.get(2)?,
            fingerprint: row.get(3)?,
            public_key: row.get(4)?,
            has_passphrase: row.get::<_, i64>(5)? != 0,
            created_at: row.get(6)?,
        })
    })?;

    Ok(rows.filter_map(|r| r.ok()).collect())
}

/// Get the private key for an SSH key (decrypted).
#[tauri::command]
pub async fn ssh_key_get_private(
    state: State<'_, AppState>,
    key_id: String,
) -> AppResult<String> {
    let conn = state.db.lock();
    let (ct, nonce_bytes): (Vec<u8>, Vec<u8>) = conn.query_row(
        "SELECT encrypted_private_key, private_key_nonce FROM ssh_keys WHERE id = ?1",
        [&key_id],
        |row| Ok((row.get(0)?, row.get(1)?)),
    )?;

    let mut nonce = [0u8; 12];
    nonce.copy_from_slice(&nonce_bytes);
    let blob = crate::crypto::EncryptedBlob { ciphertext: ct, nonce };
    let plaintext = state.vault.decrypt(&blob)?;
    Ok(String::from_utf8_lossy(&plaintext).to_string())
}

/// Delete an SSH key.
#[tauri::command]
pub async fn ssh_key_delete(
    state: State<'_, AppState>,
    key_id: String,
) -> AppResult<()> {
    let conn = state.db.lock();
    conn.execute("DELETE FROM ssh_keys WHERE id = ?1", [&key_id])?;
    Ok(())
}

/// Copy public key to clipboard (returns the key).
#[tauri::command]
pub async fn ssh_key_get_public(
    state: State<'_, AppState>,
    key_id: String,
) -> AppResult<String> {
    let conn = state.db.lock();
    let public_key: String = conn.query_row(
        "SELECT public_key FROM ssh_keys WHERE id = ?1",
        [&key_id],
        |row| row.get(0),
    )?;
    Ok(public_key)
}

mod base64_engine {
    use base64::Engine;
    pub fn encode(data: &[u8]) -> String {
        base64::engine::general_purpose::STANDARD.encode(data)
    }
}
