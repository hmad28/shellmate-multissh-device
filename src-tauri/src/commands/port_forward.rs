use crate::errors::AppResult;
use crate::port_forward::{PortForwardRule, PortForwardType};
use crate::state::AppState;
use serde::Deserialize;
use std::sync::Arc;
use tauri::{AppHandle, State};

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PortForwardCreateInput {
    pub session_id: String,
    pub rule_type: PortForwardType,
    pub local_port: u16,
    pub remote_host: String,
    pub remote_port: u16,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PortForwardIdInput {
    pub forward_id: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PortForwardListInput {
    pub session_id: String,
}

#[tauri::command]
pub async fn port_forward_create(
    app: AppHandle,
    state: State<'_, AppState>,
    input: PortForwardCreateInput,
) -> AppResult<PortForwardRule> {
    let mgr = Arc::clone(&state.port_forward);
    mgr.create_forward(
        app,
        input.session_id,
        input.rule_type,
        input.local_port,
        input.remote_host,
        input.remote_port,
    )
    .await
}

#[tauri::command]
pub async fn port_forward_list(
    state: State<'_, AppState>,
    input: PortForwardListInput,
) -> AppResult<Vec<PortForwardRule>> {
    let mgr = Arc::clone(&state.port_forward);
    Ok(mgr.list_forwards(&input.session_id))
}

#[tauri::command]
pub async fn port_forward_remove(
    state: State<'_, AppState>,
    input: PortForwardIdInput,
) -> AppResult<()> {
    let mgr = Arc::clone(&state.port_forward);
    mgr.remove_forward(&input.forward_id)
}

#[tauri::command]
pub async fn port_forward_toggle(
    state: State<'_, AppState>,
    input: PortForwardIdInput,
) -> AppResult<PortForwardRule> {
    let mgr = Arc::clone(&state.port_forward);
    mgr.toggle_forward(&input.forward_id).await
}
