use crate::audit::{AuditEvent, AuditEventType, AuditLog, AuditQuery, AuditSettings};
use crate::errors::AppResult;
use crate::state::AppState;
use tauri::State;

#[tauri::command]
pub async fn audit_record(
    state: State<'_, AppState>,
    event_type: String,
    host_id: Option<String>,
    payload: String,
) -> AppResult<String> {
    let etype = AuditEventType::from_str(&event_type).ok_or_else(|| {
        crate::errors::AppError::InvalidInput(format!("unknown event type: {event_type}"))
    })?;
    let conn = state.db.lock();
    AuditLog::record(&conn, &state.vault, etype, host_id.as_deref(), &payload)
}

#[tauri::command]
pub async fn audit_query(
    state: State<'_, AppState>,
    filter: AuditQuery,
) -> AppResult<Vec<AuditEvent>> {
    let conn = state.db.lock();
    AuditLog::query(&conn, &state.vault, &filter)
}

#[tauri::command]
pub async fn audit_export(state: State<'_, AppState>, filter: AuditQuery) -> AppResult<String> {
    let conn = state.db.lock();
    AuditLog::export_jsonl(&conn, &state.vault, &filter)
}

#[tauri::command]
pub async fn audit_purge(state: State<'_, AppState>) -> AppResult<u32> {
    let conn = state.db.lock();
    AuditLog::purge(&conn)
}

#[tauri::command]
pub async fn audit_get_settings(
    state: State<'_, AppState>,
    host_id: String,
) -> AppResult<Option<AuditSettings>> {
    let conn = state.db.lock();
    AuditLog::get_settings(&conn, &host_id)
}

#[tauri::command]
pub async fn audit_set_settings(
    state: State<'_, AppState>,
    settings: AuditSettings,
) -> AppResult<()> {
    let conn = state.db.lock();
    AuditLog::set_settings(&conn, &settings)
}
