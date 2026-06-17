use crate::errors::AppResult;
use crate::state::AppState;
use crate::sync::{SyncResult, SyncStatus};
use tauri::State;

#[tauri::command]
pub async fn sync_status(state: State<'_, AppState>) -> AppResult<SyncStatus> {
    let conn = state.db.lock();
    state.sync.status(&conn)
}

#[tauri::command]
pub async fn sync_configure(
    state: State<'_, AppState>,
    backend_type: String,
    endpoint_url: String,
    credentials: String,
) -> AppResult<()> {
    let conn = state.db.lock();
    state.sync.configure(
        &conn,
        &state.vault,
        &backend_type,
        &endpoint_url,
        &credentials,
    )
}

#[tauri::command]
pub async fn sync_now(state: State<'_, AppState>) -> AppResult<SyncResult> {
    state.sync.sync_now(&state).await
}

#[tauri::command]
pub async fn sync_pause(state: State<'_, AppState>) -> AppResult<()> {
    let conn = state.db.lock();
    state.sync.set_enabled(&conn, false)
}

#[tauri::command]
pub async fn sync_resume(state: State<'_, AppState>) -> AppResult<()> {
    let conn = state.db.lock();
    state.sync.set_enabled(&conn, true)
}
