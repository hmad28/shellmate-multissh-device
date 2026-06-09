// Prevents additional console window on Windows in release.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod db;
mod errors;
mod state;

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
            commands::system::app_version,
            commands::host::get_hosts,
            commands::host::create_host,
            commands::host::update_host,
            commands::host::delete_host,
            commands::settings::get_settings,
            commands::settings::set_setting,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
