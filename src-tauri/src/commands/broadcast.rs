use crate::state::AppState;
use tauri::State;

#[tauri::command]
pub async fn broadcast_add(
    state: State<'_, AppState>,
    session_id: String,
) -> Result<(), String> {
    state.broadcast.add_to_broadcast(&session_id);
    Ok(())
}

#[tauri::command]
pub async fn broadcast_remove(
    state: State<'_, AppState>,
    session_id: String,
) -> Result<(), String> {
    state.broadcast.remove_from_broadcast(&session_id);
    Ok(())
}

#[tauri::command]
pub async fn broadcast_is_active(
    state: State<'_, AppState>,
    session_id: String,
) -> Result<bool, String> {
    Ok(state.broadcast.is_broadcasting(&session_id))
}

#[tauri::command]
pub async fn broadcast_get_sessions(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    Ok(state.broadcast.get_broadcast_sessions())
}

#[tauri::command]
pub async fn broadcast_send(
    state: State<'_, AppState>,
    session_id: String,
    data: String,
) -> Result<(), String> {
    state
        .broadcast
        .broadcast_input(&session_id, data.into_bytes())
        .map_err(|e| e.to_string())
}
