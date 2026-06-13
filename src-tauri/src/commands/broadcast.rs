use crate::errors::AppResult;
use crate::state::AppState;
use tauri::State;

#[tauri::command]
pub async fn broadcast_add(
    state: State<'_, AppState>,
    session_id: String,
) -> AppResult<()> {
    state.broadcast.add_to_broadcast(&session_id);
    Ok(())
}

#[tauri::command]
pub async fn broadcast_remove(
    state: State<'_, AppState>,
    session_id: String,
) -> AppResult<()> {
    state.broadcast.remove_from_broadcast(&session_id);
    Ok(())
}

#[tauri::command]
pub async fn broadcast_is_active(
    state: State<'_, AppState>,
    session_id: String,
) -> AppResult<bool> {
    Ok(state.broadcast.is_broadcasting(&session_id))
}

#[tauri::command]
pub async fn broadcast_get_sessions(state: State<'_, AppState>) -> AppResult<Vec<String>> {
    Ok(state.broadcast.get_broadcast_sessions())
}

#[tauri::command]
pub async fn broadcast_send(
    state: State<'_, AppState>,
    session_id: String,
    data: String,
) -> AppResult<()> {
    state
        .broadcast
        .broadcast_input(&session_id, data.into_bytes())
        .map_err(|e| crate::errors::AppError::Internal(e.to_string()))
}
