use crate::ssh::SessionManager;
use crate::vault::Vault;
use parking_lot::Mutex;
use rusqlite::Connection;
use std::sync::Arc;

/// Shared application state managed by Tauri.
pub struct AppState {
    pub db: Arc<Mutex<Connection>>,
    pub vault: Arc<Vault>,
    pub ssh: Arc<SessionManager>,
}

impl AppState {
    pub fn new(db: Connection) -> Self {
        Self {
            db: Arc::new(Mutex::new(db)),
            vault: Arc::new(Vault::new()),
            ssh: Arc::new(SessionManager::new()),
        }
    }
}
