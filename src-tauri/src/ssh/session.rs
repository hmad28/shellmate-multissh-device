use crate::errors::{AppError, AppResult};
use crate::known_hosts::KnownHostsManager;
use crate::ssh::handler::ClientHandler;
use crate::state::AppState;
use parking_lot::Mutex as PlMutex;
use russh::client;
use russh::keys::decode_secret_key;
use russh::ChannelMsg;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tauri::{AppHandle, Emitter};
use tokio::sync::mpsc;
use uuid::Uuid;

/// Hard limit per process. Prevents runaway resource usage from rogue UI.
const MAX_SESSIONS: usize = 50;

/// Soft warning threshold. Frontend may surface this in the UI.
pub const SOFT_SESSION_LIMIT: usize = 20;

/// Outbound (frontend-bound) event names.
const EVENT_OUTPUT_PREFIX: &str = "ssh:output:";
const EVENT_STATUS_PREFIX: &str = "ssh:status:";
const EVENT_ERROR_PREFIX: &str = "ssh:error:";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectParams {
    pub hostname: String,
    pub port: u16,
    pub username: String,
    pub auth: AuthMaterial,
    pub label: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AuthMaterial {
    Password { password: String },
    PrivateKey { private_key: String, passphrase: Option<String> },
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionStatus {
    Connecting,
    Connected,
    Disconnected,
    Failed,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StatusEvent {
    pub session_id: String,
    pub status: SessionStatus,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OutputEvent {
    pub session_id: String,
    /// UTF-8-lossy decoded chunk. xterm.js handles raw ANSI escape codes.
    pub data: String,
}

/// One active SSH session = one TCP connection + one shell channel.
/// Per docs/04-backend-plan §9: 1-connection-per-tab strategy.
struct Session {
    /// Outbound channel: send keystrokes / resize commands here.
    outbound: mpsc::UnboundedSender<OutboundMsg>,
}

enum OutboundMsg {
    Data(Vec<u8>),
    Resize { cols: u32, rows: u32 },
    Close,
}

pub struct SessionManager {
    sessions: PlMutex<HashMap<String, Session>>,
    known_hosts: Arc<KnownHostsManager>,
}

impl SessionManager {
    pub fn new(known_hosts: Arc<KnownHostsManager>) -> Self {
        Self {
            sessions: PlMutex::new(HashMap::new()),
            known_hosts,
        }
    }

    pub fn count(&self) -> usize {
        self.sessions.lock().len()
    }

    /// Open a new SSH session. Spawns an async task that owns the russh
    /// connection and forwards I/O via Tauri events.
    pub async fn open(
        self: &Arc<Self>,
        app: AppHandle,
        params: ConnectParams,
    ) -> AppResult<String> {
        if self.count() >= MAX_SESSIONS {
            return Err(AppError::InvalidInput(format!(
                "session limit reached ({MAX_SESSIONS})"
            )));
        }

        let session_id = Uuid::new_v4().to_string();
        let (tx, rx) = mpsc::unbounded_channel::<OutboundMsg>();

        // Register before spawning so the frontend can reference the id immediately.
        self.sessions
            .lock()
            .insert(session_id.clone(), Session { outbound: tx });

        let mgr = Arc::clone(self);
        let app_for_task = app.clone();
        let session_id_for_task = session_id.clone();

        tokio::spawn(async move {
            emit_status(
                &app_for_task,
                &session_id_for_task,
                SessionStatus::Connecting,
                None,
            );

            let res = run_session(
                app_for_task.clone(),
                session_id_for_task.clone(),
                params,
                rx,
                Arc::clone(&mgr.known_hosts),
            )
            .await;

            // Cleanup registered session handles
            {
                use tauri::Manager;
                if let Some(state) = app_for_task.try_state::<AppState>() {
                    state.sftp.cleanup_ssh_session(&session_id_for_task);
                    state.port_forward.cleanup_session(&session_id_for_task);
                }
            }

            if let Err(e) = res {
                log::warn!("ssh session {session_id_for_task} ended with error: {e}");
                emit_status(
                    &app_for_task,
                    &session_id_for_task,
                    SessionStatus::Failed,
                    Some(e.to_string()),
                );
                emit_error(&app_for_task, &session_id_for_task, &e.to_string());
            } else {
                emit_status(
                    &app_for_task,
                    &session_id_for_task,
                    SessionStatus::Disconnected,
                    None,
                );
            }
            mgr.sessions.lock().remove(&session_id_for_task);
        });

        Ok(session_id)
    }

    pub fn send_input(&self, session_id: &str, data: Vec<u8>) -> AppResult<()> {
        let sessions = self.sessions.lock();
        let session = sessions
            .get(session_id)
            .ok_or_else(|| AppError::NotFound(format!("session {session_id}")))?;
        session
            .outbound
            .send(OutboundMsg::Data(data))
            .map_err(|_| AppError::Internal("session channel closed".into()))?;
        Ok(())
    }

    pub fn resize(&self, session_id: &str, cols: u32, rows: u32) -> AppResult<()> {
        let sessions = self.sessions.lock();
        let session = sessions
            .get(session_id)
            .ok_or_else(|| AppError::NotFound(format!("session {session_id}")))?;
        session
            .outbound
            .send(OutboundMsg::Resize { cols, rows })
            .map_err(|_| AppError::Internal("session channel closed".into()))?;
        Ok(())
    }

    pub fn close(&self, session_id: &str) -> AppResult<()> {
        let session = self.sessions.lock().remove(session_id);
        if let Some(session) = session {
            // Best-effort: signal task to stop; ignore if already closed.
            let _ = session.outbound.send(OutboundMsg::Close);
            Ok(())
        } else {
            Err(AppError::NotFound(format!("session {session_id}")))
        }
    }

    pub fn close_all(&self) {
        let mut sessions = self.sessions.lock();
        for (_, session) in sessions.drain() {
            let _ = session.outbound.send(OutboundMsg::Close);
        }
    }
}

/// Owns the russh client + channel. Drives I/O until disconnect.
async fn run_session(
    app: AppHandle,
    session_id: String,
    params: ConnectParams,
    mut rx: mpsc::UnboundedReceiver<OutboundMsg>,
    known_hosts: Arc<KnownHostsManager>,
) -> AppResult<()> {
    let config = client::Config {
        inactivity_timeout: None, // disabled; we use keepalive instead
        keepalive_interval: Some(Duration::from_secs(60)),
        keepalive_max: 3,
        ..Default::default()
    };

    let handler = ClientHandler::new(
        known_hosts,
        params.hostname.clone(),
        params.port,
        app.clone(),
        session_id.clone(),
    );

    let mut handle = client::connect(
        Arc::new(config),
        (params.hostname.as_str(), params.port),
        handler,
    )
    .await
    .map_err(|e| AppError::Internal(format!("ssh connect failed: {e}")))?;

    let auth_ok = match &params.auth {
        AuthMaterial::Password { password } => {
            handle
                .authenticate_password(&params.username, password.clone())
                .await
                .map_err(|e| AppError::Internal(format!("ssh auth error: {e}")))?
        }
        AuthMaterial::PrivateKey {
            private_key,
            passphrase,
        } => {
            let key = decode_secret_key(private_key, passphrase.as_deref())
                .map_err(|e| AppError::InvalidInput(format!("invalid private key: {e}")))?;
            handle
                .authenticate_publickey(&params.username, Arc::new(key))
                .await
                .map_err(|e| AppError::Internal(format!("ssh auth error: {e}")))?
        }
    };

    if !auth_ok {
        return Err(AppError::InvalidInput(
            "authentication failed (check credentials)".into(),
        ));
    }

    let mut channel = handle
        .channel_open_session()
        .await
        .map_err(|e| AppError::Internal(format!("open channel failed: {e}")))?;

    // Request a PTY so the remote shell behaves interactively.
    channel
        .request_pty(false, "xterm-256color", 80, 24, 0, 0, &[])
        .await
        .map_err(|e| AppError::Internal(format!("request pty failed: {e}")))?;

    channel
        .request_shell(false)
        .await
        .map_err(|e| AppError::Internal(format!("request shell failed: {e}")))?;

    let handle = Arc::new(handle);

    // Register parameters for SFTP and Port Forwarding
    {
        use tauri::Manager;
        let state = app.state::<AppState>();
        state.sftp.register_ssh_session(&session_id, params.clone());
        state.port_forward.register_ssh_handle(&session_id, Arc::clone(&handle));
    }

    emit_status(&app, &session_id, SessionStatus::Connected, None);

    // Main I/O loop. Two select arms:
    //   - inbound from server channel  -> emit to frontend as `ssh:output:{id}`
    //   - outbound from frontend       -> write to channel
    loop {
        tokio::select! {
            msg = rx.recv() => {
                match msg {
                    Some(OutboundMsg::Data(bytes)) => {
                        if channel.data(bytes.as_slice()).await.is_err() {
                            break;
                        }
                    }
                    Some(OutboundMsg::Resize { cols, rows }) => {
                        let _ = channel.window_change(cols, rows, 0, 0).await;
                    }
                    Some(OutboundMsg::Close) | None => break,
                }
            }
            ev = channel.wait() => {
                match ev {
                    Some(ChannelMsg::Data { data }) => {
                        let chunk = String::from_utf8_lossy(&data).to_string();
                        emit_output(&app, &session_id, chunk);
                    }
                    Some(ChannelMsg::ExtendedData { data, .. }) => {
                        // stderr (extended data type 1)
                        let chunk = String::from_utf8_lossy(&data).to_string();
                        emit_output(&app, &session_id, chunk);
                    }
                    Some(ChannelMsg::ExitStatus { exit_status }) => {
                        log::info!("session {session_id} remote exit_status={exit_status}");
                    }
                    Some(ChannelMsg::Eof) | Some(ChannelMsg::Close) | None => break,
                    _ => {}
                }
            }
        }
    }

    let _ = channel.close().await;
    let _ = handle
        .disconnect(russh::Disconnect::ByApplication, "client closing", "en")
        .await;
    Ok(())
}

fn emit_status(app: &AppHandle, session_id: &str, status: SessionStatus, message: Option<String>) {
    let event = StatusEvent {
        session_id: session_id.to_string(),
        status,
        message,
    };
    let _ = app.emit(&format!("{EVENT_STATUS_PREFIX}{session_id}"), &event);
}

fn emit_output(app: &AppHandle, session_id: &str, data: String) {
    let event = OutputEvent {
        session_id: session_id.to_string(),
        data,
    };
    let _ = app.emit(&format!("{EVENT_OUTPUT_PREFIX}{session_id}"), &event);
}

fn emit_error(app: &AppHandle, session_id: &str, message: &str) {
    let _ = app.emit(
        &format!("{EVENT_ERROR_PREFIX}{session_id}"),
        serde_json::json!({ "sessionId": session_id, "message": message }),
    );
}
