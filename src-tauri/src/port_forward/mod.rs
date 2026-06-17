use crate::errors::{AppError, AppResult};
use crate::ssh::handler::ClientHandler;
use parking_lot::Mutex as PlMutex;
use russh::client::Handle;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PortForwardRule {
    pub id: String,
    pub session_id: String,
    pub rule_type: PortForwardType,
    pub local_port: u16,
    pub remote_host: String,
    pub remote_port: u16,
    pub enabled: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PortForwardType {
    Local,
    Remote,
}

struct ActiveForward {
    rule: PortForwardRule,
    shutdown_tx: tokio::sync::oneshot::Sender<()>,
    enabled: Arc<tokio::sync::RwLock<bool>>,
}

pub struct PortForwardManager {
    forwards: PlMutex<HashMap<String, ActiveForward>>,
    ssh_handles: PlMutex<HashMap<String, Arc<Handle<ClientHandler>>>>,
}

impl PortForwardManager {
    pub fn new() -> Self {
        Self {
            forwards: PlMutex::new(HashMap::new()),
            ssh_handles: PlMutex::new(HashMap::new()),
        }
    }

    pub fn register_ssh_handle(&self, session_id: &str, handle: Arc<Handle<ClientHandler>>) {
        self.ssh_handles
            .lock()
            .insert(session_id.to_string(), handle);
    }

    pub async fn create_forward(
        &self,
        app: tauri::AppHandle,
        session_id: String,
        rule_type: PortForwardType,
        local_port: u16,
        remote_host: String,
        remote_port: u16,
    ) -> AppResult<PortForwardRule> {
        let handle = {
            let handles = self.ssh_handles.lock();
            handles
                .get(&session_id)
                .ok_or_else(|| AppError::NotFound(format!("SSH session {}", session_id)))?
                .clone()
        };

        let rule = PortForwardRule {
            id: Uuid::new_v4().to_string(),
            session_id,
            rule_type,
            local_port,
            remote_host,
            remote_port,
            enabled: true,
        };

        let addr = SocketAddr::from(([127, 0, 0, 1], local_port));
        let listener = TcpListener::bind(addr).await.map_err(|e| {
            AppError::Internal(format!(
                "Failed to bind local port {}: {} (port may be in use)",
                local_port, e
            ))
        })?;

        let (shutdown_tx, mut shutdown_rx) = tokio::sync::oneshot::channel::<()>();
        let enabled = Arc::new(tokio::sync::RwLock::new(true));
        let enabled_for_task = enabled.clone();

        let rule_for_task = rule.clone();
        let handle_for_task = handle;

        tokio::spawn(async move {
            loop {
                tokio::select! {
                    accept_result = listener.accept() => {
                        // Skip connections when disabled
                        if !*enabled_for_task.read().await {
                            continue;
                        }
                        match accept_result {
                            Ok((mut local_stream, _)) => {
                                let handle = handle_for_task.clone();
                                let rule = rule_for_task.clone();
                                tokio::spawn(async move {
                                    let channel = match handle
                                        .channel_open_direct_tcpip(
                                            &rule.remote_host,
                                            rule.remote_port.into(),
                                            "127.0.0.1",
                                            local_port.into(),
                                        )
                                        .await
                                    {
                                        Ok(ch) => ch,
                                        Err(e) => {
                                            log::warn!("Failed to open SSH channel for port forward: {}", e);
                                            return;
                                        }
                                    };

                                    let mut channel_stream = channel.into_stream();
                                    if let Err(e) = tokio::io::copy_bidirectional(&mut local_stream, &mut channel_stream).await {
                                        log::debug!("Port forward stream ended: {}", e);
                                    }
                                });
                            }
                            Err(e) => {
                                log::warn!("Failed to accept connection on port {}: {}", local_port, e);
                            }
                        }
                    }
                    _ = &mut shutdown_rx => {
                        log::info!("Port forward on {} shutting down", local_port);
                        break;
                    }
                }
            }
        });

        self.forwards.lock().insert(
            rule.id.clone(),
            ActiveForward {
                rule: rule.clone(),
                shutdown_tx,
                enabled,
            },
        );

        Ok(rule)
    }

    pub fn list_forwards(&self, session_id: &str) -> Vec<PortForwardRule> {
        self.forwards
            .lock()
            .values()
            .filter(|f| f.rule.session_id == session_id)
            .map(|f| f.rule.clone())
            .collect()
    }

    pub fn remove_forward(&self, forward_id: &str) -> AppResult<()> {
        let mut forwards = self.forwards.lock();
        if let Some(active) = forwards.remove(forward_id) {
            let _ = active.shutdown_tx.send(());
            Ok(())
        } else {
            Err(AppError::NotFound(format!("forward {}", forward_id)))
        }
    }

    pub async fn toggle_forward(&self, forward_id: &str) -> AppResult<PortForwardRule> {
        let (enabled_val, enabled_arc, rule) = {
            let mut forwards = self.forwards.lock();
            let active = forwards
                .get_mut(forward_id)
                .ok_or_else(|| AppError::NotFound(format!("forward {}", forward_id)))?;

            active.rule.enabled = !active.rule.enabled;
            (
                active.rule.enabled,
                active.enabled.clone(),
                active.rule.clone(),
            )
        };

        // Update the shared enabled flag so the spawned task respects it
        *enabled_arc.write().await = enabled_val;

        Ok(rule)
    }

    pub fn cleanup_session(&self, session_id: &str) {
        let mut forwards = self.forwards.lock();
        let ids_to_remove: Vec<String> = forwards
            .iter()
            .filter(|(_, f)| f.rule.session_id == session_id)
            .map(|(id, _)| id.clone())
            .collect();

        for id in ids_to_remove {
            if let Some(active) = forwards.remove(&id) {
                let _ = active.shutdown_tx.send(());
            }
        }

        self.ssh_handles.lock().remove(session_id);
    }
}

impl Default for PortForwardManager {
    fn default() -> Self {
        Self::new()
    }
}
