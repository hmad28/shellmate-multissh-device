// Prevents additional console window on Windows in release.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod crypto;
mod db;
mod errors;
mod known_hosts;
mod plugin;
mod port_forward;
mod sftp;
mod ssh;
mod state;
mod vault;
mod biometric;
mod sync;
mod team;
mod audit;
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
        .plugin(tauri_plugin_updater::Builder::new().build())
        .setup(|app| {
            let app_data_dir = app
                .path()
                .app_data_dir()
                .expect("failed to resolve app data dir");
            let db_path = app_data_dir.join("shellmate.db");

            log::info!("Opening database at {}", db_path.display());

            // Check if vault metadata exists — if so, DB is encrypted.
            // Don't try to open it without a key. The unlock flow will handle it.
            let conn = if db::has_vault_meta(&db_path) {
                log::info!("Vault metadata found — DB is encrypted. Deferring open until unlock.");
                // Create a temporary in-memory DB so AppState can be initialized.
                // The real DB will be swapped in after vault unlock.
                db::open(&std::path::Path::new(":memory:"), None)
                    .expect("failed to create temporary database")
            } else {
                db::open(&db_path, None).expect("failed to initialize database")
            };

            app.manage(AppState::new(conn, db_path));
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
            // Git
            commands::git::git_get_info,
            // Command History
            commands::history::history_add,
            commands::history::history_list,
            commands::history::history_search,
            commands::history::history_clear,
            // Biometric
            commands::biometric::biometric_status,
            commands::biometric::biometric_enable,
            commands::biometric::biometric_disable,
            commands::biometric::biometric_unlock,
            // Sync
            commands::sync::sync_status,
            commands::sync::sync_configure,
            commands::sync::sync_now,
            commands::sync::sync_pause,
            commands::sync::sync_resume,
            // Team Vault
            commands::team::team_create,
            commands::team::team_list,
            commands::team::team_delete,
            commands::team::team_add_member,
            commands::team::team_list_members,
            commands::team::team_revoke_member,
            commands::team::team_share_host,
            commands::team::team_list_shares,
            commands::team::team_remove_share,
            // Plugin System
            commands::plugin::plugin_list,
            commands::plugin::plugin_install,
            commands::plugin::plugin_uninstall,
            commands::plugin::plugin_enable,
            commands::plugin::plugin_disable,
            commands::plugin::plugin_get_capabilities,
            commands::plugin::plugin_grant_capability,
            commands::plugin::plugin_revoke_capability,
            commands::plugin::plugin_execute,
            // Audit Log
            commands::audit::audit_record,
            commands::audit::audit_query,
            commands::audit::audit_export,
            commands::audit::audit_purge,
            commands::audit::audit_get_settings,
            commands::audit::audit_set_settings,
            // Export/Import
            commands::export::export_hosts_encrypted,
            commands::export::import_hosts_encrypted,
            // Server Stats & Remote Exec
            commands::server_stats::server_stats_exec,
            commands::server_stats::remote_exec,
            // Session Recording
            commands::recording::recording_start,
            commands::recording::recording_stop,
            commands::recording::recording_event,
            commands::recording::recording_list,
            commands::recording::recording_events,
            commands::recording::recording_delete,
            // SSH Key Manager
            commands::ssh_key::ssh_key_generate,
            commands::ssh_key::ssh_key_list,
            commands::ssh_key::ssh_key_get_private,
            commands::ssh_key::ssh_key_get_public,
            commands::ssh_key::ssh_key_delete,
            // Local Shell
            commands::local_shell::local_shell_spawn,
            commands::local_shell::local_shell_send,
            commands::local_shell::local_shell_read,
            commands::local_shell::local_shell_kill,
            commands::local_shell::local_shell_list,
            // Host-to-Host Transfer
            commands::host_transfer::sftp_host_transfer,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
