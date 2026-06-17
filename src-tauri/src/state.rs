use crate::biometric::BiometricProvider;
use crate::known_hosts::KnownHostsManager;
use crate::plugin::runtime::PluginRuntime;
use crate::port_forward::PortForwardManager;
use crate::sftp::SftpManager;
use crate::ssh::BroadcastManager;
use crate::ssh::SessionManager;
use crate::sync::SyncEngine;
use crate::vault::Vault;
use dashmap::DashMap;
use parking_lot::Mutex;
use rusqlite::Connection;
use std::path::PathBuf;
use std::sync::Arc;

#[cfg(not(any(target_os = "android", target_os = "ios")))]
pub struct LocalSessionState {
    pub child: Box<dyn portable_pty::Child + Send>,
    pub master: Box<dyn portable_pty::MasterPty + Send>,
    pub writer: Box<dyn std::io::Write + Send>,
}

#[cfg(any(target_os = "android", target_os = "ios"))]
pub struct LocalSessionState;

pub struct AppState {
    pub db: Arc<Mutex<Connection>>,
    pub db_path: PathBuf,
    pub vault: Arc<Vault>,
    pub ssh: Arc<SessionManager>,
    pub sftp: Arc<SftpManager>,
    pub port_forward: Arc<PortForwardManager>,
    pub known_hosts: Arc<KnownHostsManager>,
    pub broadcast: Arc<BroadcastManager>,
    pub biometric: Arc<Box<dyn BiometricProvider>>,
    pub sync: Arc<SyncEngine>,
    pub plugin_runtime: Arc<PluginRuntime>,
    pub local_sessions: Arc<DashMap<String, tokio::sync::Mutex<LocalSessionState>>>,
    pub local_session_output: Arc<DashMap<String, tokio::sync::Mutex<String>>>,
}

impl AppState {
    pub fn new(db: Connection, db_path: PathBuf) -> Self {
        let db_arc = Arc::new(Mutex::new(db));
        let known_hosts = Arc::new(KnownHostsManager::new(Arc::clone(&db_arc)));
        let plugin_runtime =
            Arc::new(PluginRuntime::new().expect("failed to initialize plugin runtime"));
        Self {
            db: Arc::clone(&db_arc),
            db_path,
            vault: Arc::new(Vault::new()),
            ssh: Arc::new(SessionManager::new(Arc::clone(&known_hosts))),
            sftp: Arc::new(SftpManager::new()),
            port_forward: Arc::new(PortForwardManager::new()),
            known_hosts,
            broadcast: Arc::new(BroadcastManager::new()),
            biometric: Arc::new(crate::biometric::create_provider()),
            sync: Arc::new(SyncEngine::new()),
            plugin_runtime,
            local_sessions: Arc::new(DashMap::new()),
            local_session_output: Arc::new(DashMap::new()),
        }
    }

    /// Replace the database connection. Used after SQLCipher migration or
    /// vault unlock to swap in an encrypted connection.
    pub fn swap_db(&self, new_conn: Connection) {
        let mut db = self.db.lock();
        *db = new_conn;
    }
}
