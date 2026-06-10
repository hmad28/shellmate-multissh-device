use crate::crypto::EncryptedBlob;
use crate::errors::{AppError, AppResult};
use crate::state::AppState;
use chrono::Utc;
use serde::Deserialize;
use tauri::State;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CredentialType {
    Password,
    PrivateKey,
}

impl CredentialType {
    fn as_str(&self) -> &'static str {
        match self {
            CredentialType::Password => "password",
            CredentialType::PrivateKey => "private_key",
        }
    }
}

/// Save a credential. The plaintext is encrypted with the unlocked vault key
/// before being persisted. Returns the credential id.
#[tauri::command]
pub async fn save_credential(
    state: State<'_, AppState>,
    cred_type: CredentialType,
    plaintext: String,
) -> AppResult<String> {
    if plaintext.is_empty() {
        return Err(AppError::InvalidInput("credential plaintext is empty".into()));
    }

    let blob = state.vault.encrypt(plaintext.as_bytes())?;
    let id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();

    let conn = state.db.lock();
    conn.execute(
        "INSERT INTO credentials (id, type, encrypted_data, nonce, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        rusqlite::params![
            id,
            cred_type.as_str(),
            blob.ciphertext,
            blob.nonce.to_vec(),
            now,
            now,
        ],
    )?;
    state.vault.record_activity();
    Ok(id)
}

/// Internal helper: load and decrypt a credential. Used by SSH module — not
/// exposed as a Tauri command to avoid leaking plaintext to the frontend.
pub fn load_credential_plaintext(state: &AppState, id: &str) -> AppResult<Vec<u8>> {
    let (ct, nonce_bytes): (Vec<u8>, Vec<u8>) = {
        let conn = state.db.lock();
        conn.query_row(
            "SELECT encrypted_data, nonce FROM credentials WHERE id = ?1",
            [id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => {
                AppError::NotFound(format!("credential {id}"))
            }
            other => AppError::Database(other),
        })?
    };

    if nonce_bytes.len() != 12 {
        return Err(AppError::Internal("nonce length mismatch".into()));
    }
    let mut nonce = [0u8; 12];
    nonce.copy_from_slice(&nonce_bytes);

    let blob = EncryptedBlob {
        ciphertext: ct,
        nonce,
    };
    state.vault.decrypt(&blob)
}

#[tauri::command]
pub async fn delete_credential(
    state: State<'_, AppState>,
    id: String,
) -> AppResult<()> {
    let conn = state.db.lock();
    let deleted = conn.execute("DELETE FROM credentials WHERE id = ?1", [&id])?;
    if deleted == 0 {
        return Err(AppError::NotFound(format!("credential {id}")));
    }
    Ok(())
}
