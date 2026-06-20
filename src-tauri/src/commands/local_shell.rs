use crate::errors::AppResult;
use crate::state::AppState;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::{Emitter, State};
use std::sync::OnceLock;
use dashmap::DashMap;
use tokio::sync::mpsc;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum StreamMessage {
    Output { data: String },
    Status { status: String, message: Option<String> },
    Error { message: String },
}

pub static TERMINAL_STREAMS: OnceLock<DashMap<String, mpsc::UnboundedSender<StreamMessage>>> = OnceLock::new();

pub fn get_terminal_streams() -> &'static DashMap<String, mpsc::UnboundedSender<StreamMessage>> {
    TERMINAL_STREAMS.get_or_init(DashMap::new)
}


#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LocalSession {
    pub id: String,
    pub shell: String,
    pub pid: u32,
}

#[cfg(not(any(target_os = "android", target_os = "ios")))]
use std::io::{Read, Write};

#[cfg(not(any(target_os = "android", target_os = "ios")))]
/// Spawn a local shell process.
#[tauri::command]
pub async fn local_shell_spawn(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    shell: Option<String>,
) -> AppResult<LocalSession> {
    spawn_local_shell(
        app,
        Arc::clone(&state.local_sessions),
        Arc::clone(&state.local_session_output),
        shell,
    )
    .await
}

#[cfg(not(any(target_os = "android", target_os = "ios")))]
pub async fn spawn_local_shell(
    app: tauri::AppHandle,
    local_sessions: Arc<
        dashmap::DashMap<String, tokio::sync::Mutex<crate::state::LocalSessionState>>,
    >,
    output_buffers: Arc<dashmap::DashMap<String, tokio::sync::Mutex<String>>>,
    shell: Option<String>,
) -> AppResult<LocalSession> {
    let session_id = uuid::Uuid::new_v4().to_string();

    #[cfg(target_os = "windows")]
    let (cmd, args): (String, Vec<String>) = {
        let shell = shell.unwrap_or_else(|| {
            if std::path::Path::new(
                "C:\\Windows\\System32\\WindowsPowerShell\\v1.0\\powershell.exe",
            )
            .exists()
            {
                "powershell".to_string()
            } else {
                "cmd".to_string()
            }
        });
        match shell.as_str() {
            "powershell" | "pwsh" => ("powershell.exe".to_string(), vec!["-NoLogo".to_string()]),
            "cmd" => ("cmd.exe".to_string(), vec![]),
            "git-bash" => (
                "C:\\Program Files\\Git\\bin\\bash.exe".to_string(),
                vec!["--login".to_string()],
            ),
            "wsl" => ("wsl.exe".to_string(), vec![]),
            _ => {
                return Err(crate::errors::AppError::InvalidInput(format!(
                    "unsupported local shell: {shell}"
                )));
            }
        }
    };

    #[cfg(not(target_os = "windows"))]
    let (cmd, args): (String, Vec<String>) = {
        let shell = shell
            .unwrap_or_else(|| std::env::var("SHELL").unwrap_or_else(|_| "/bin/bash".to_string()));
        match shell.as_str() {
            "/bin/bash" | "/bin/zsh" | "/bin/fish" | "/bin/sh" => (shell, vec![]),
            _ => {
                return Err(crate::errors::AppError::InvalidInput(format!(
                    "unsupported local shell: {shell}"
                )));
            }
        }
    };

    let pty_system = portable_pty::native_pty_system();
    let pair = pty_system
        .openpty(portable_pty::PtySize {
            rows: 24,
            cols: 80,
            pixel_width: 0,
            pixel_height: 0,
        })
        .map_err(|e| crate::errors::AppError::Internal(format!("openpty: {e}")))?;

    let mut cmd_builder = portable_pty::CommandBuilder::new(&cmd);
    cmd_builder.args(&args);

    let child = pair
        .slave
        .spawn_command(cmd_builder)
        .map_err(|e| crate::errors::AppError::Internal(format!("spawn {cmd}: {e}")))?;

    drop(pair.slave);

    let mut reader = pair.master.try_clone_reader().map_err(|e| {
        crate::errors::AppError::Internal(format!("failed to clone master reader: {e}"))
    })?;
    let writer = pair.master.take_writer().map_err(|e| {
        crate::errors::AppError::Internal(format!("failed to take master writer: {e}"))
    })?;

    // Spawn reader thread for stdout/stderr
    let app_stdout = app.clone();
    let session_id_stdout = session_id.clone();
    let output_buffers_stdout = Arc::clone(&output_buffers);
    std::thread::spawn(move || {
        let mut read_buf = vec![0u8; 4096];
        let mut carry_over = Vec::new();
        loop {
            match reader.read(&mut read_buf) {
                Ok(0) => {
                    if !carry_over.is_empty() {
                        let text = String::from_utf8_lossy(&carry_over).into_owned();
                        append_local_output(&output_buffers_stdout, &session_id_stdout, &text);
                        let _ = app_stdout.emit(
                            &format!("ssh:output:{}", session_id_stdout),
                            serde_json::json!({
                                "sessionId": session_id_stdout,
                                "data": text,
                            }),
                        );
                        if let Some(sender) = get_terminal_streams().get(&session_id_stdout) {
                            let _ = sender.send(StreamMessage::Output { data: text });
                        }
                    }
                    break; // EOF
                }
                Ok(n) => {
                    let mut combined = carry_over;
                    combined.extend_from_slice(&read_buf[..n]);

                    let (valid, invalid) = split_valid_utf8(&combined);
                    if !valid.is_empty() {
                        let text = String::from_utf8_lossy(valid).into_owned();
                        append_local_output(&output_buffers_stdout, &session_id_stdout, &text);
                        let _ = app_stdout.emit(
                            &format!("ssh:output:{}", session_id_stdout),
                            serde_json::json!({
                                "sessionId": session_id_stdout,
                                "data": text,
                            }),
                        );
                        if let Some(sender) = get_terminal_streams().get(&session_id_stdout) {
                            let _ = sender.send(StreamMessage::Output { data: text });
                        }
                    }
                    carry_over = invalid.to_vec();
                }
                Err(_) => break,
            }
        }
    });

    let session_state = crate::state::LocalSessionState {
        child,
        master: pair.master,
        writer,
    };

    // Store the child process handle.
    local_sessions.insert(session_id.clone(), tokio::sync::Mutex::new(session_state));
    output_buffers.insert(session_id.clone(), tokio::sync::Mutex::new(String::new()));

    // Spawn status monitoring task
    let app_status = app.clone();
    let session_id_status = session_id.clone();
    let local_sessions_status = Arc::clone(&local_sessions);
    let output_buffers_status = Arc::clone(&output_buffers);
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(std::time::Duration::from_millis(200)).await;

            let mut exited = false;
            if let Some(entry) = local_sessions_status.get(&session_id_status) {
                if let Ok(mut session) = entry.value().try_lock() {
                    match session.child.try_wait() {
                        Ok(Some(_status)) => {
                            exited = true;
                        }
                        Err(_) => {
                            exited = true;
                        }
                        _ => {}
                    }
                }
            } else {
                exited = true;
            }

            if exited {
                let _ = app_status.emit(
                    &format!("ssh:status:{}", session_id_status),
                    serde_json::json!({
                        "sessionId": session_id_status,
                        "status": "disconnected",
                        "message": null,
                    }),
                );
                if let Some(sender) = get_terminal_streams().get(&session_id_status) {
                    let _ = sender.send(StreamMessage::Status { status: "disconnected".to_string(), message: None });
                }
                local_sessions_status.remove(&session_id_status);
                output_buffers_status.remove(&session_id_status);
                get_terminal_streams().remove(&session_id_status);
                break;
            }
        }
    });

    // Emit connected status immediately
    let _ = app.emit(
        &format!("ssh:status:{}", session_id),
        serde_json::json!({
            "sessionId": session_id,
            "status": "connected",
            "message": null,
        }),
    );

    Ok(LocalSession {
        id: session_id,
        shell: cmd,
        pid: 0,
    })
}

#[cfg(not(any(target_os = "android", target_os = "ios")))]
fn append_local_output(
    output_buffers: &dashmap::DashMap<String, tokio::sync::Mutex<String>>,
    session_id: &str,
    text: &str,
) {
    if let Some(entry) = output_buffers.get(session_id) {
        if let Ok(mut buffer) = entry.value().try_lock() {
            buffer.push_str(text);
        }
    }
}

#[cfg(not(any(target_os = "android", target_os = "ios")))]
/// Send data to a local shell.
#[tauri::command]
pub async fn local_shell_send(
    state: State<'_, AppState>,
    session_id: String,
    data: String,
) -> AppResult<()> {
    send_local_shell(Arc::clone(&state.local_sessions), session_id, data).await
}

#[cfg(not(any(target_os = "android", target_os = "ios")))]
pub async fn send_local_shell(
    local_sessions: Arc<
        dashmap::DashMap<String, tokio::sync::Mutex<crate::state::LocalSessionState>>,
    >,
    session_id: String,
    data: String,
) -> AppResult<()> {
    let entry = local_sessions
        .get(&session_id)
        .ok_or_else(|| crate::errors::AppError::NotFound("session not found".into()))?;
    let mut session = entry.value().lock().await;
    session
        .writer
        .write_all(data.as_bytes())
        .map_err(|e| crate::errors::AppError::Internal(format!("write: {e}")))?;
    session
        .writer
        .flush()
        .map_err(|e| crate::errors::AppError::Internal(format!("flush: {e}")))?;
    Ok(())
}

#[cfg(not(any(target_os = "android", target_os = "ios")))]
/// Read output from a local shell (non-blocking, returns available data).
#[tauri::command]
pub async fn local_shell_read(state: State<'_, AppState>, session_id: String) -> AppResult<String> {
    read_local_shell(Arc::clone(&state.local_session_output), session_id).await
}

#[cfg(not(any(target_os = "android", target_os = "ios")))]
pub async fn read_local_shell(
    output_buffers: Arc<dashmap::DashMap<String, tokio::sync::Mutex<String>>>,
    session_id: String,
) -> AppResult<String> {
    let Some(entry) = output_buffers.get(&session_id) else {
        return Ok(String::new());
    };
    let mut buffer = entry.value().lock().await;
    let output = buffer.clone();
    buffer.clear();
    Ok(output)
}

#[cfg(not(any(target_os = "android", target_os = "ios")))]
/// Kill a local shell session.
#[tauri::command]
pub async fn local_shell_kill(state: State<'_, AppState>, session_id: String) -> AppResult<()> {
    kill_local_shell(
        Arc::clone(&state.local_sessions),
        Arc::clone(&state.local_session_output),
        session_id,
    )
    .await
}

#[cfg(not(any(target_os = "android", target_os = "ios")))]
pub async fn kill_local_shell(
    local_sessions: Arc<
        dashmap::DashMap<String, tokio::sync::Mutex<crate::state::LocalSessionState>>,
    >,
    output_buffers: Arc<dashmap::DashMap<String, tokio::sync::Mutex<String>>>,
    session_id: String,
) -> AppResult<()> {
    if let Some((_, session)) = local_sessions.remove(&session_id) {
        let mut session = session.lock().await;
        session.child.kill().ok();
    }
    output_buffers.remove(&session_id);
    Ok(())
}

#[cfg(not(any(target_os = "android", target_os = "ios")))]
/// Resize the local shell terminal.
#[tauri::command]
pub async fn local_shell_resize(
    state: State<'_, AppState>,
    session_id: String,
    cols: u16,
    rows: u16,
) -> AppResult<()> {
    resize_local_shell(Arc::clone(&state.local_sessions), session_id, cols, rows).await
}

#[cfg(not(any(target_os = "android", target_os = "ios")))]
pub async fn resize_local_shell(
    local_sessions: Arc<
        dashmap::DashMap<String, tokio::sync::Mutex<crate::state::LocalSessionState>>,
    >,
    session_id: String,
    cols: u16,
    rows: u16,
) -> AppResult<()> {
    let entry = local_sessions
        .get(&session_id)
        .ok_or_else(|| crate::errors::AppError::NotFound("session not found".into()))?;
    let session = entry.value().lock().await;
    session
        .master
        .resize(portable_pty::PtySize {
            rows,
            cols,
            pixel_width: 0,
            pixel_height: 0,
        })
        .map_err(|e| crate::errors::AppError::Internal(format!("resize: {e}")))?;
    Ok(())
}

#[cfg(not(any(target_os = "android", target_os = "ios")))]
/// List available local shells.
#[tauri::command]
pub async fn local_shell_list() -> AppResult<Vec<String>> {
    #[cfg(target_os = "windows")]
    {
        let mut shells = vec!["cmd".to_string(), "powershell".to_string()];
        if std::path::Path::new("C:\\Program Files\\PowerShell\\7\\pwsh.exe").exists() {
            shells.push("pwsh".to_string());
        }
        if std::path::Path::new("C:\\Program Files\\Git\\bin\\bash.exe").exists() {
            shells.push("git-bash".to_string());
        }
        if which::which("wsl.exe").is_ok() {
            shells.push("wsl".to_string());
        }
        Ok(shells)
    }

    #[cfg(not(target_os = "windows"))]
    {
        let mut shells = Vec::new();
        for shell in &["/bin/bash", "/bin/zsh", "/bin/fish", "/bin/sh"] {
            if std::path::Path::new(shell).exists() {
                shells.push(shell.to_string());
            }
        }
        Ok(shells)
    }
}

#[cfg(not(any(target_os = "android", target_os = "ios")))]
fn split_valid_utf8(bytes: &[u8]) -> (&[u8], &[u8]) {
    let len = bytes.len();
    let max_back = std::cmp::min(len, 4);
    for i in 1..=max_back {
        let index = len - i;
        let byte = bytes[index];
        if byte >= 0xC0 {
            let expected_len = if byte >= 0xF0 {
                4
            } else if byte >= 0xE0 {
                3
            } else {
                2
            };
            if i < expected_len {
                return (&bytes[..index], &bytes[index..]);
            } else {
                break;
            }
        } else if byte < 0x80 {
            break;
        }
    }
    (bytes, &[])
}

// --- Mobile Target Stubs ---

#[cfg(any(target_os = "android", target_os = "ios"))]
#[tauri::command]
pub async fn local_shell_spawn(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    shell: Option<String>,
) -> AppResult<LocalSession> {
    let mut body = serde_json::Map::new();
    if let Some(shell) = shell {
        body.insert("shell".into(), serde_json::json!(shell));
    }
    let value =
        crate::commands::p2p_sync::p2p_post_desktop_terminal(&state, "/terminal/spawn", body)
            .await?;
    let mut session: LocalSession = serde_json::from_value(
        value
            .get("session")
            .cloned()
            .ok_or_else(|| crate::errors::AppError::Internal("missing desktop session".into()))?,
    )
    .map_err(|e| crate::errors::AppError::Internal(format!("invalid desktop session: {e}")))?;
    
    let original_id = session.id.clone();
    session.id = format!("desktop:{}", original_id);
    
    // Initialize the local buffer for output fallback
    state.local_session_output.insert(session.id.clone(), tokio::sync::Mutex::new(String::new()));

    // Spawn the background persistent stream reader task
    crate::commands::p2p_sync::start_desktop_terminal_stream(
        app,
        state.db.clone(),
        state.local_session_output.clone(),
        session.id.clone(),
    );

    Ok(session)
}

#[cfg(any(target_os = "android", target_os = "ios"))]
#[tauri::command]
pub async fn local_shell_send(
    state: State<'_, AppState>,
    session_id: String,
    data: String,
) -> AppResult<()> {
    let mut body = serde_json::Map::new();
    body.insert(
        "sessionId".into(),
        serde_json::json!(session_id.trim_start_matches("desktop:")),
    );
    body.insert("data".into(), serde_json::json!(data));
    crate::commands::p2p_sync::p2p_post_desktop_terminal(&state, "/terminal/send", body).await?;
    Ok(())
}

#[cfg(any(target_os = "android", target_os = "ios"))]
#[tauri::command]
pub async fn local_shell_read(state: State<'_, AppState>, session_id: String) -> AppResult<String> {
    if let Some(entry) = state.local_session_output.get(&session_id) {
        let mut buffer = entry.value().lock().await;
        let data = buffer.clone();
        buffer.clear();
        Ok(data)
    } else {
        Ok(String::new())
    }
}

#[cfg(any(target_os = "android", target_os = "ios"))]
#[tauri::command]
pub async fn local_shell_kill(state: State<'_, AppState>, session_id: String) -> AppResult<()> {
    let mut body = serde_json::Map::new();
    body.insert(
        "sessionId".into(),
        serde_json::json!(session_id.trim_start_matches("desktop:")),
    );
    crate::commands::p2p_sync::p2p_post_desktop_terminal(&state, "/terminal/kill", body).await?;
    Ok(())
}

#[cfg(any(target_os = "android", target_os = "ios"))]
#[tauri::command]
pub async fn local_shell_resize(
    state: State<'_, AppState>,
    session_id: String,
    cols: u16,
    rows: u16,
) -> AppResult<()> {
    let mut body = serde_json::Map::new();
    body.insert(
        "sessionId".into(),
        serde_json::json!(session_id.trim_start_matches("desktop:")),
    );
    body.insert("cols".into(), serde_json::json!(cols));
    body.insert("rows".into(), serde_json::json!(rows));
    crate::commands::p2p_sync::p2p_post_desktop_terminal(&state, "/terminal/resize", body).await?;
    Ok(())
}

#[cfg(any(target_os = "android", target_os = "ios"))]
#[tauri::command]
pub async fn local_shell_list() -> AppResult<Vec<String>> {
    Ok(vec![])
}
