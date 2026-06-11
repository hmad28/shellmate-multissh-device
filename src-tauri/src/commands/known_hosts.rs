use crate::known_hosts::manager::{KnownHost, HostKeyVerificationResult};
use crate::state::AppState;
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrustHostInput {
    pub hostname: String,
    pub port: u16,
    pub key_type: String,
    pub public_key: Vec<u8>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VerifyHostKeyInput {
    pub hostname: String,
    pub port: u16,
    pub key_type: String,
    pub public_key: Vec<u8>,
}

#[tauri::command]
pub fn known_hosts_verify(
    state: State<AppState>,
    input: VerifyHostKeyInput,
) -> Result<HostKeyVerificationResult, String> {
    state
        .known_hosts
        .verify_host_key(&input.hostname, input.port, &input.key_type, &input.public_key)
        .map_err(|e| format!("Failed to verify host key: {}", e))
}

#[tauri::command]
pub fn known_hosts_list(state: State<AppState>) -> Result<Vec<KnownHost>, String> {
    state
        .known_hosts
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
        .trust_host_key(&input.hostname, input.port, &input.key_type, &input.public_key)
        .map_err(|e| format!("Failed to trust host: {}", e))
}

#[tauri::command]
pub fn known_hosts_set_trusted(
    state: State<AppState>,
    id: String,
    trusted: bool,
) -> Result<(), String> {
    state
        .known_hosts
        .update_trust(&id, trusted)
        .map_err(|e| format!("Failed to update trust status: {}", e))
}

#[tauri::command]
pub fn known_hosts_remove(state: State<AppState>, id: String) -> Result<(), String> {
    state
        .known_hosts
        .remove(&id)
        .map_err(|e| format!("Failed to remove host: {}", e))
}
