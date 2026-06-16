use crate::commands::credential::load_credential_plaintext;
use crate::errors::{AppError, AppResult};
use crate::ssh::session::{AuthMaterial, ConnectParams};
use crate::state::AppState;
use serde::Deserialize;
use std::sync::Arc;
use tauri::{AppHandle, State};

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectByHostInput {
    pub host_id: String,
    pub session_id: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum QuickConnectAuth {
    #[serde(rename = "password")]
    Password { password: String },
    #[serde(rename = "key")]
    Key {
        private_key: String,
        passphrase: Option<String>,
    },
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuickConnectInput {
    pub hostname: String,
    pub port: u16,
    pub username: String,
    pub label: Option<String>,
    pub auth: QuickConnectAuth,
    pub shell: Option<String>,
    pub session_id: Option<String>,
}

/// Open an SSH session for a saved host. Decrypts the credential via the
/// vault (must be unlocked) and starts a session task.
#[tauri::command]
pub async fn ssh_connect(
    app: AppHandle,
    state: State<'_, AppState>,
    input: ConnectByHostInput,
) -> AppResult<String> {
    if !state.vault.is_unlocked() {
        return Err(AppError::InvalidInput("vault is locked".into()));
    }

    // Load host row
    let (hostname, port, username, auth_type, credential_id, label): (
        String,
        i64,
        String,
        String,
        String,
        String,
    ) = {
        let conn = state.db.lock();
        conn.query_row(
            "SELECT hostname, port, username, auth_type, credential_id, label
             FROM hosts WHERE id = ?1",
            [&input.host_id],
            |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                    row.get(5)?,
                ))
            },
        )
        .map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => {
                AppError::NotFound(format!("host {}", input.host_id))
            }
            other => AppError::Database(other),
        })?
    };

    let plaintext = load_credential_plaintext(&state, &credential_id)?;
    let auth = match auth_type.as_str() {
        "password" => AuthMaterial::Password {
            password: String::from_utf8(plaintext)
                .map_err(|_| AppError::Internal("password not valid UTF-8".into()))?,
        },
        "key" | "key_passphrase" => AuthMaterial::PrivateKey {
            private_key: String::from_utf8(plaintext)
                .map_err(|_| AppError::Internal("private key not valid UTF-8".into()))?,
            passphrase: None,
        },
        other => {
            return Err(AppError::InvalidInput(format!(
                "unsupported auth_type: {other}"
            )))
        }
    };

    let params = ConnectParams {
        hostname,
        port: port as u16,
        username,
        auth,
        label: Some(label),
        shell: None,
    };

    let session_id = input.session_id.unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
    let mgr = Arc::clone(&state.ssh);
    state.vault.record_activity();
    mgr.open(app, params, session_id).await
}

/// Open an SSH session without saving the credential. Useful for testing /
/// one-off connections during MVP development.
#[tauri::command]
pub async fn ssh_quick_connect(
    app: AppHandle,
    state: State<'_, AppState>,
    input: QuickConnectInput,
) -> AppResult<String> {
    let auth = match input.auth {
        QuickConnectAuth::Password { password } => AuthMaterial::Password { password },
        QuickConnectAuth::Key {
            private_key,
            passphrase,
        } => AuthMaterial::PrivateKey {
            private_key,
            passphrase,
        },
    };
    let params = ConnectParams {
        hostname: input.hostname,
        port: input.port,
        username: input.username,
        auth,
        label: input.label,
        shell: input.shell,
    };
    let session_id = input.session_id.unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
    let mgr = Arc::clone(&state.ssh);
    mgr.open(app, params, session_id).await
}

#[tauri::command]
pub async fn ssh_send(
    state: State<'_, AppState>,
    session_id: String,
    data: String,
) -> AppResult<()> {
    if state.local_sessions.contains_key(&session_id) {
        crate::commands::local_shell::local_shell_send(state, session_id, data).await
    } else {
        state.ssh.send_input(&session_id, data.into_bytes())
    }
}

#[tauri::command]
pub async fn ssh_resize(
    state: State<'_, AppState>,
    session_id: String,
    cols: u32,
    rows: u32,
) -> AppResult<()> {
    if state.local_sessions.contains_key(&session_id) {
        Ok(())
    } else {
        state.ssh.resize(&session_id, cols, rows)
    }
}

#[tauri::command]
pub async fn ssh_disconnect(
    state: State<'_, AppState>,
    session_id: String,
) -> AppResult<()> {
    if state.local_sessions.contains_key(&session_id) {
        crate::commands::local_shell::local_shell_kill(state, session_id).await
    } else {
        state.ssh.close(&session_id)
    }
}
