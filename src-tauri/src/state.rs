use parking_lot::Mutex;
use rusqlite::Connection;
use std::sync::Arc;

/// Shared application state managed by Tauri.
pub struct AppState {
    pub db: Arc<Mutex<Connection>>,
}

impl AppState {
    pub fn new(db: Connection) -> Self {
        Self {
            db: Arc::new(Mutex::new(db)),
        }
    }
}
