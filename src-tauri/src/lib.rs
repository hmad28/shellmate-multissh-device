// Prevents additional console window on Windows in release.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod crypto;
mod db;
mod errors;
mod known_hosts;
mod port_forward;
mod sftp;
mod ssh;
mod state;
mod vault;

use crate::commands::p2p_sync::SyncServerState;
use crate::commands::vip_access::VipKeyStore;
use crate::state::AppState;
use std::sync::Arc;
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
            app.manage(Arc::new(VipKeyStore::new()));
            app.manage(Arc::new(SyncServerState::new()));
            commands::discovery::init(app);
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
            commands::host::search_hosts,
            // Groups
            commands::group::get_groups,
            commands::group::create_group,
            commands::group::update_group,
            commands::group::delete_group,
            commands::group::move_host_to_group,
            // Snippets
            commands::snippet::get_snippets,
            commands::snippet::create_snippet,
            commands::snippet::update_snippet,
            commands::snippet::delete_snippet,
            // Themes
            commands::theme::get_themes,
            commands::theme::save_theme,
            commands::theme::delete_theme,
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
            commands::vault::vault_change_master_password,
            // Credentials
            commands::credential::save_credential,
            commands::credential::delete_credential,
            // SSH
            commands::ssh::ssh_connect,
            commands::ssh::ssh_quick_connect,
            commands::ssh::ssh_send,
            commands::ssh::ssh_resize,
            commands::ssh::ssh_disconnect,
            // SFTP
            commands::sftp::sftp_open,
            commands::sftp::sftp_list,
            commands::sftp::sftp_upload,
            commands::sftp::sftp_download,
            commands::sftp::sftp_rename,
            commands::sftp::sftp_remove,
            commands::sftp::sftp_mkdir,
            commands::sftp::sftp_close,
            // Port Forwarding
            commands::port_forward::port_forward_create,
            commands::port_forward::port_forward_list,
            commands::port_forward::port_forward_remove,
            commands::port_forward::port_forward_toggle,
            // Known Hosts
            commands::known_hosts::known_hosts_verify,
            commands::known_hosts::known_hosts_trust,
            commands::known_hosts::known_hosts_list,
            commands::known_hosts::known_hosts_remove,
            commands::known_hosts::known_hosts_set_trusted,
            // Broadcast
            commands::broadcast::broadcast_add,
            commands::broadcast::broadcast_remove,
            commands::broadcast::broadcast_is_active,
            commands::broadcast::broadcast_get_sessions,
            commands::broadcast::broadcast_send,
            // Discovery
            commands::discovery::start_discovery,
            commands::discovery::stop_discovery,
            commands::discovery::start_broadcasting,
            // VIP Access
            commands::vip_access::vip_generate_keypair,
            commands::vip_access::vip_inject_authorized_keys,
            commands::vip_access::vip_create_localhost_host,
            commands::vip_access::vip_get_key_status,
            // P2P Sync
            commands::p2p_sync::p2p_start_sync_server,
            commands::p2p_sync::p2p_stop_sync_server,
            commands::p2p_sync::p2p_get_sync_status,
            commands::p2p_sync::p2p_export_for_sync,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
