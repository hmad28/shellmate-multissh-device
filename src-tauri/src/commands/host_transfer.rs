use crate::errors::AppResult;
use crate::state::AppState;
use serde::{Deserialize, Serialize};
use tauri::{Emitter, State};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SftpTransfer {
    pub id: String,
    pub source_host: String,
    pub source_path: String,
    pub dest_host: String,
    pub dest_path: String,
    pub status: String,
    pub bytes_transferred: u64,
    pub total_bytes: u64,
    pub error: Option<String>,
}

/// Transfer a file between two remote hosts.
/// This is a scaffold — full implementation requires SFTP client library integration.
#[tauri::command]
pub async fn sftp_host_transfer(
    _state: State<'_, AppState>,
    source_host_id: String,
    source_path: String,
    dest_host_id: String,
    dest_path: String,
    app_handle: tauri::AppHandle,
) -> AppResult<SftpTransfer> {
    let transfer_id = uuid::Uuid::new_v4().to_string();

    let transfer = SftpTransfer {
        id: transfer_id.clone(),
        source_host: source_host_id,
        source_path,
        dest_host: dest_host_id,
        dest_path,
        status: "pending".to_string(),
        bytes_transferred: 0,
        total_bytes: 0,
        error: Some("Host-to-host transfer is not yet implemented. Use local SFTP browser to download from source and upload to destination.".to_string()),
    };

    let _ = app_handle.emit("sftp:transfer:progress", &transfer);

    Ok(transfer)
}
