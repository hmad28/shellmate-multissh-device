use crate::errors::AppResult;
use crate::plugin::manifest::PluginManifest;
use crate::plugin::{Plugin, PluginCapability, PluginManager};
use crate::state::AppState;
use std::sync::Arc;
use tauri::State;

#[tauri::command]
pub async fn plugin_list(state: State<'_, AppState>) -> AppResult<Vec<Plugin>> {
    let conn = state.db.lock();
    PluginManager::list(&conn)
}

#[tauri::command]
pub async fn plugin_install(
    state: State<'_, AppState>,
    manifest_json: String,
    wasm_path: String,
) -> AppResult<Plugin> {
    let manifest: PluginManifest = serde_json::from_str(&manifest_json)
        .map_err(|e| crate::errors::AppError::InvalidInput(format!("invalid manifest: {e}")))?;

    crate::plugin::manifest::validate_manifest(&manifest)
        .map_err(crate::errors::AppError::InvalidInput)?;

    // Validate WASM file.
    state.plugin_runtime.validate(&wasm_path)?;

    // Copy WASM to app plugin directory.
    let plugin_dir = state.db_path.parent().unwrap_or(std::path::Path::new("."));
    let plugins_dir = plugin_dir.join("plugins");
    std::fs::create_dir_all(&plugins_dir)?;

    let dest_filename = format!("{}.wasm", manifest.id.replace('.', "_"));
    let dest_path = plugins_dir.join(&dest_filename);
    std::fs::copy(&wasm_path, &dest_path)?;

    let dest_str = dest_path.to_string_lossy().to_string();

    let conn = state.db.lock();
    PluginManager::install(&conn, &manifest, &dest_str)
}

#[tauri::command]
pub async fn plugin_uninstall(state: State<'_, AppState>, plugin_id: String) -> AppResult<()> {
    let conn = state.db.lock();
    PluginManager::uninstall(&conn, &plugin_id)
}

#[tauri::command]
pub async fn plugin_enable(state: State<'_, AppState>, plugin_id: String) -> AppResult<()> {
    let conn = state.db.lock();
    PluginManager::set_enabled(&conn, &plugin_id, true)
}

#[tauri::command]
pub async fn plugin_disable(state: State<'_, AppState>, plugin_id: String) -> AppResult<()> {
    let conn = state.db.lock();
    PluginManager::set_enabled(&conn, &plugin_id, false)
}

#[tauri::command]
pub async fn plugin_get_capabilities(
    state: State<'_, AppState>,
    plugin_id: String,
) -> AppResult<Vec<PluginCapability>> {
    let conn = state.db.lock();
    PluginManager::list_capabilities(&conn, &plugin_id)
}

#[tauri::command]
pub async fn plugin_grant_capability(
    state: State<'_, AppState>,
    plugin_id: String,
    capability: String,
) -> AppResult<()> {
    let conn = state.db.lock();
    PluginManager::set_capability(&conn, &plugin_id, &capability, true)
}

#[tauri::command]
pub async fn plugin_revoke_capability(
    state: State<'_, AppState>,
    plugin_id: String,
    capability: String,
) -> AppResult<()> {
    let conn = state.db.lock();
    PluginManager::set_capability(&conn, &plugin_id, &capability, false)
}

#[tauri::command]
pub async fn plugin_execute(state: State<'_, AppState>, plugin_id: String) -> AppResult<String> {
    let capabilities = {
        let conn = state.db.lock();
        let plugin = PluginManager::get(&conn, &plugin_id)?;
        if !plugin.enabled {
            return Err(crate::errors::AppError::InvalidInput(
                "plugin is disabled".into(),
            ));
        }
        PluginManager::list_capabilities(&conn, &plugin_id)?
    };

    let missing: Vec<String> = capabilities
        .into_iter()
        .filter(|cap| !cap.granted)
        .map(|cap| cap.capability)
        .collect();
    if !missing.is_empty() {
        return Err(crate::errors::AppError::InvalidInput(format!(
            "plugin has ungranted capabilities: {}",
            missing.join(", ")
        )));
    }

    let wasm_path = {
        let conn = state.db.lock();
        conn.query_row(
            "SELECT wasm_path FROM plugins WHERE id = ?1",
            [&plugin_id],
            |row| row.get::<_, String>(0),
        )
        .map_err(|_| crate::errors::AppError::NotFound("plugin wasm_path".into()))?
    };

    let runtime = Arc::clone(&state.plugin_runtime);
    let result = tokio::task::spawn_blocking(move || runtime.execute(&wasm_path))
        .await
        .map_err(|e| crate::errors::AppError::Internal(format!("join: {e}")))?;

    result
}
