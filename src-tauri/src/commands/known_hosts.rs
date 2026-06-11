use crate::known_hosts::manager::{KnownHost, KnownHostsManager};
use crate::state::AppState;
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Debug, Deserialize)]
pub struct TrustHostInput {
    pub hostname: String,
    pub port: u16,
    pub key_type: String,
    pub public_key_blob: Vec<u8>,
}

#[derive(Debug, Deserialize)]
pub struct SetTrustedInput {
    pub id: String,
    pub trusted: bool,
}

#[derive(Debug, Deserialize)]
pub struct RemoveHostInput {
    pub id: String,
}

#[tauri::command]
pub fn known_hosts_list(state: State<AppState>) -> Result<Vec<KnownHost>, String> {
    state
        .known_hosts_manager
        .list()
        .map_err(|e| format!("Failed to list known hosts: {}", e))
}

#[tauri::command]
pub fn known_hosts_trust(
    state: State<AppState>,
    input: TrustHostInput,
) -> Result<String, String> {
    state
        .known_hosts
        .trust_host_key(&input.hostname, input.port, &input.key_type, &input.public_key_blob)
        .map_err(|e| format!("Failed to trust host: {}", e))
}

#[tauri::command]
pub fn known_hosts_set_trusted(
    state: State<AppState>,
    input: SetTrustedInput,
) -> Result<(), String> {
    state
        .known_hosts_manager
        .update_trust(&input.id, input.trusted)
        .map_err(|e| format!("Failed to update trust status: {}", e))
}

#[tauri::command]
pub fn known_hosts_remove(state: State<AppState>, input: RemoveHostInput) -> Result<(), String> {
    state
        .known_hosts_manager
        .remove(&input.id)
        .map_err(|e| format!("Failed to remove host: {}", e))
}
