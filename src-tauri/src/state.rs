use crate::known_hosts::KnownHostsManager;
use crate::port_forward::PortForwardManager;
use crate::sftp::SftpManager;
use crate::ssh::BroadcastManager;
use crate::ssh::SessionManager;
use crate::vault::Vault;
use parking_lot::Mutex;
use rusqlite::Connection;
use std::sync::Arc;

pub struct AppState {
    pub db: Arc<Mutex<Connection>>,
    pub vault: Arc<Vault>,
    pub ssh: Arc<SessionManager>,
    pub sftp: Arc<SftpManager>,
    pub port_forward: Arc<PortForwardManager>,
    pub known_hosts: Arc<KnownHostsManager>,
    pub broadcast: Arc<BroadcastManager>,
}

impl AppState {
    pub fn new(db: Connection) -> Self {
        let db_arc = Arc::new(Mutex::new(db));
        let known_hosts = Arc::new(KnownHostsManager::new(Arc::clone(&db_arc)));
        Self {
            db: Arc::clone(&db_arc),
            vault: Arc::new(Vault::new()),
            ssh: Arc::new(SessionManager::new(Arc::clone(&known_hosts))),
            sftp: Arc::new(SftpManager::new()),
            port_forward: Arc::new(PortForwardManager::new()),
            known_hosts,
            broadcast: Arc::new(BroadcastManager::new()),
        }
    }
}
