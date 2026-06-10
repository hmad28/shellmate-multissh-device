// Prevents additional console window on Windows in release.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod crypto;
mod db;
mod errors;
mod ssh;
mod state;
mod vault;

use crate::state::AppState;
use tauri::Manager;

/// Tauri application entry point. Exposed as a library for cross-platform
/// targets (mobile uses this same entry).
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            let app_data_dir = app
                .path()
                .app_data_dir()
                .expect("failed to resolve app data dir");
            let db_path = app_data_dir.join("shellmate.db");

            log::info!("Opening database at {}", db_path.display());
            let conn = db::open(&db_path).expect("failed to initialize database");

            app.manage(AppState::new(conn));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // System
            commands::system::app_version,
            // Hosts
            commands::host::get_hosts,
            commands::host::create_host,
            commands::host::update_host,
            commands::host::delete_host,
            // Settings
            commands::settings::get_settings,
            commands::settings::set_setting,
            // Vault
            commands::vault::vault_status,
            commands::vault::vault_setup,
            commands::vault::vault_unlock,
            commands::vault::vault_lock,
            commands::vault::vault_check_idle,
            commands::vault::vault_record_activity,
            // Credentials
            commands::credential::save_credential,
            commands::credential::delete_credential,
            // SSH
            commands::ssh::ssh_connect,
            commands::ssh::ssh_quick_connect,
            commands::ssh::ssh_send,
            commands::ssh::ssh_resize,
            commands::ssh::ssh_disconnect,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
