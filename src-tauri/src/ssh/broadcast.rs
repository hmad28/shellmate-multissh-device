use crate::errors::AppResult;
use parking_lot::Mutex;
use std::collections::HashSet;
use tokio::sync::broadcast;

pub struct BroadcastManager {
    broadcast_groups: Mutex<HashSet<String>>,
    sender: broadcast::Sender<BroadcastMessage>,
}

#[derive(Debug, Clone)]
pub struct BroadcastMessage {
    pub session_id: String,
    pub data: Vec<u8>,
}

impl BroadcastManager {
    pub fn new() -> Self {
        let (sender, _) = broadcast::channel(1000);
        Self {
            broadcast_groups: Mutex::new(HashSet::new()),
            sender,
        }
    }

    pub fn add_to_broadcast(&self, session_id: &str) {
        self.broadcast_groups
            .lock()
            .insert(session_id.to_string());
    }

    pub fn remove_from_broadcast(&self, session_id: &str) {
        self.broadcast_groups
            .lock()
            .remove(session_id);
    }

    pub fn is_broadcasting(&self, session_id: &str) -> bool {
        self.broadcast_groups.lock().contains(session_id)
    }

    pub fn get_broadcast_sessions(&self) -> Vec<String> {
        self.broadcast_groups.lock().iter().cloned().collect()
    }

    pub fn broadcast_input(&self, source_session_id: &str, data: Vec<u8>) -> AppResult<()> {
        let message = BroadcastMessage {
            session_id: source_session_id.to_string(),
            data,
        };
        let _ = self.sender.send(message);
        Ok(())
    }

    pub fn subscribe(&self) -> broadcast::Receiver<BroadcastMessage> {
        self.sender.subscribe()
    }
}

impl Default for BroadcastManager {
    fn default() -> Self {
        Self::new()
    }
}
