use crate::errors::AppResult;
use crate::known_hosts::manager::{HostKeyVerificationResult, KnownHost};
use crate::state::AppState;
use serde::Deserialize;
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
) -> AppResult<HostKeyVerificationResult> {
    state.known_hosts.verify_host_key(
        &input.hostname,
        input.port,
        &input.key_type,
        &input.public_key,
    )
}

#[tauri::command]
pub fn known_hosts_list(state: State<AppState>) -> AppResult<Vec<KnownHost>> {
    state.known_hosts.list()
}

#[tauri::command]
pub fn known_hosts_trust(state: State<AppState>, input: TrustHostInput) -> AppResult<String> {
    state.known_hosts.trust_host_key(
        &input.hostname,
        input.port,
        &input.key_type,
        &input.public_key,
    )
}

#[tauri::command]
pub fn known_hosts_set_trusted(state: State<AppState>, id: String, trusted: bool) -> AppResult<()> {
    state.known_hosts.update_trust(&id, trusted)
}

#[tauri::command]
pub fn known_hosts_remove(state: State<AppState>, id: String) -> AppResult<()> {
    state.known_hosts.remove(&id)
}
