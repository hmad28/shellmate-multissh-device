use crate::errors::AppResult;
use crate::sftp::SftpFile;
use crate::state::AppState;
use serde::Deserialize;
use std::path::PathBuf;
use std::sync::Arc;
use tauri::{AppHandle, State};

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SftpOpenInput {
    pub session_id: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SftpListInput {
    pub sftp_id: String,
    pub path: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SftpUploadInput {
    pub sftp_id: String,
    pub local_path: String,
    pub remote_path: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SftpDownloadInput {
    pub sftp_id: String,
    pub remote_path: String,
    pub local_path: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SftpRenameInput {
    pub sftp_id: String,
    pub old_path: String,
    pub new_path: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SftpPathInput {
    pub sftp_id: String,
    pub path: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SftpCloseInput {
    pub sftp_id: String,
}

#[tauri::command]
pub async fn sftp_open(
    app: AppHandle,
    state: State<'_, AppState>,
    input: SftpOpenInput,
) -> AppResult<String> {
    let mgr = Arc::clone(&state.sftp);
    mgr.open_sftp(app, Arc::clone(&state.known_hosts), input.session_id)
        .await
}

#[tauri::command]
pub async fn sftp_list(
    state: State<'_, AppState>,
    input: SftpListInput,
) -> AppResult<Vec<SftpFile>> {
    let mgr = Arc::clone(&state.sftp);
    mgr.list_directory(&input.sftp_id, input.path).await
}

#[tauri::command]
pub async fn sftp_upload(
    app: AppHandle,
    state: State<'_, AppState>,
    input: SftpUploadInput,
) -> AppResult<()> {
    let mgr = Arc::clone(&state.sftp);
    mgr.upload_file(
        app,
        &input.sftp_id,
        PathBuf::from(input.local_path),
        input.remote_path,
    )
    .await
}

#[tauri::command]
pub async fn sftp_download(
    app: AppHandle,
    state: State<'_, AppState>,
    input: SftpDownloadInput,
) -> AppResult<()> {
    let mgr = Arc::clone(&state.sftp);
    mgr.download_file(
        app,
        &input.sftp_id,
        input.remote_path,
        PathBuf::from(input.local_path),
    )
    .await
}

#[tauri::command]
pub async fn sftp_rename(state: State<'_, AppState>, input: SftpRenameInput) -> AppResult<()> {
    let mgr = Arc::clone(&state.sftp);
    mgr.rename(&input.sftp_id, input.old_path, input.new_path)
        .await
}

#[tauri::command]
pub async fn sftp_remove(state: State<'_, AppState>, input: SftpPathInput) -> AppResult<()> {
    let mgr = Arc::clone(&state.sftp);
    mgr.remove(&input.sftp_id, input.path).await
}

#[tauri::command]
pub async fn sftp_mkdir(state: State<'_, AppState>, input: SftpPathInput) -> AppResult<()> {
    let mgr = Arc::clone(&state.sftp);
    mgr.mkdir(&input.sftp_id, input.path).await
}

#[tauri::command]
pub async fn sftp_close(state: State<'_, AppState>, input: SftpCloseInput) -> AppResult<()> {
    let mgr = Arc::clone(&state.sftp);
    mgr.close(&input.sftp_id)
}
