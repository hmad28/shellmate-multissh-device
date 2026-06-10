use crate::errors::AppResult;
use crate::state::AppState;
use crate::vault::Vault;
use serde::Serialize;
use tauri::State;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VaultStatus {
    pub initialized: bool,
    pub unlocked: bool,
}

#[tauri::command]
pub async fn vault_status(state: State<'_, AppState>) -> AppResult<VaultStatus> {
    let initialized = {
        let conn = state.db.lock();
        Vault::is_initialized(&conn)?
    };
    Ok(VaultStatus {
        initialized,
        unlocked: state.vault.is_unlocked(),
    })
}

#[tauri::command]
pub async fn vault_setup(
    state: State<'_, AppState>,
    password: String,
) -> AppResult<()> {
    let conn = state.db.lock();
    state.vault.setup(&conn, &password)
}

#[tauri::command]
pub async fn vault_unlock(
    state: State<'_, AppState>,
    password: String,
) -> AppResult<()> {
    let conn = state.db.lock();
    state.vault.unlock(&conn, &password)
}

#[tauri::command]
pub async fn vault_lock(state: State<'_, AppState>) -> AppResult<()> {
    state.vault.lock();
    Ok(())
}

#[tauri::command]
pub async fn vault_check_idle(state: State<'_, AppState>) -> AppResult<bool> {
    Ok(state.vault.check_idle())
}

#[tauri::command]
pub async fn vault_record_activity(state: State<'_, AppState>) -> AppResult<()> {
    state.vault.record_activity();
    Ok(())
}
