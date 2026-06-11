use crate::errors::AppResult;
use crate::known_hosts::KnownHostsManager;
use crate::ssh::handler::ClientHandler;
use crate::ssh::session::{AuthMaterial, ConnectParams};
use russh::client;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::watch;

/// Reconnect backoff schedule: 1s, 2s, 5s, 10s, 30s, 60s (max)
const BACKOFF_SCHEDULE: &[u64] = &[1, 2, 5, 10, 30, 60];

pub struct ReconnectHandle {
    cancel_tx: watch::Sender<bool>,
}

impl ReconnectHandle {
    pub fn cancel(&self) {
        let _ = self.cancel_tx.send(true);
    }
}

pub fn spawn_reconnect(
    app: tauri::AppHandle,
    session_id: String,
    params: ConnectParams,
    known_hosts: Arc<KnownHostsManager>,
    on_success: impl Fn(client::Handle<ClientHandler>) + Send + 'static,
    on_status: impl Fn(ReconnectStatus) + Send + 'static,
) -> ReconnectHandle {
    let (cancel_tx, cancel_rx) = watch::channel(false);

    tokio::spawn(async move {
        let mut attempt = 0;
        let mut rx = cancel_rx;

        loop {
            let delay = if attempt < BACKOFF_SCHEDULE.len() {
                BACKOFF_SCHEDULE[attempt]
            } else {
                *BACKOFF_SCHEDULE.last().unwrap()
            };

            on_status(ReconnectStatus::Waiting {
                attempt: attempt + 1,
                delay_secs: delay,
            });

            // Wait for delay or cancellation
            tokio::select! {
                _ = tokio::time::sleep(Duration::from_secs(delay)) => {}
                _ = rx.changed() => {
                    if *rx.borrow() {
                        on_status(ReconnectStatus::Cancelled);
                        return;
                    }
                }
            }

            on_status(ReconnectStatus::Connecting {
                attempt: attempt + 1,
            });

            // Attempt reconnection
            let config = client::Config {
                inactivity_timeout: Some(Duration::from_secs(0)),
                keepalive_interval: Some(Duration::from_secs(60)),
                keepalive_max: 3,
                ..Default::default()
            };

            let handler = ClientHandler::new(
                Arc::clone(&known_hosts),
                params.hostname.clone(),
                params.port,
                app.clone(),
                session_id.clone(),
            );

            match client::connect(
                Arc::new(config),
                (params.hostname.as_str(), params.port),
                handler,
            )
            .await
            {
                Ok(mut handle) => {
                    // Authenticate
                    let auth_result = match &params.auth {
                        AuthMaterial::Password { password } => {
                            handle
                                .authenticate_password(&params.username, password.clone())
                                .await
                        }
                        AuthMaterial::PrivateKey {
                            private_key,
                            passphrase,
                        } => {
                            match russh::keys::decode_secret_key(private_key, passphrase.as_deref())
                            {
                                Ok(key) => {
                                    handle
                                        .authenticate_publickey(&params.username, Arc::new(key))
                                        .await
                                }
                                Err(e) => {
                                    on_status(ReconnectStatus::Failed {
                                        error: format!("Invalid private key: {}", e),
                                    });
                                    return;
                                }
                            }
                        }
                    };

                    match auth_result {
                        Ok(true) => {
                            on_status(ReconnectStatus::Connected);
                            on_success(handle);
                            return;
                        }
                        Ok(false) => {
                            on_status(ReconnectStatus::Failed {
                                error: "Authentication failed".to_string(),
                            });
                            return;
                        }
                        Err(e) => {
                            log::warn!("Reconnect attempt {} failed: {}", attempt + 1, e);
                            attempt += 1;
                            continue;
                        }
                    }
                }
                Err(e) => {
                    log::warn!("Reconnect attempt {} failed: {}", attempt + 1, e);
                    attempt += 1;
                    continue;
                }
            }
        }
    });

    ReconnectHandle { cancel_tx }
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ReconnectStatus {
    Waiting { attempt: usize, delay_secs: u64 },
    Connecting { attempt: usize },
    Connected,
    Failed { error: String },
    Cancelled,
}
