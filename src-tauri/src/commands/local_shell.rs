use crate::errors::AppResult;
use crate::state::AppState;
use serde::{Deserialize, Serialize};
use std::process::Stdio;
use tauri::{Emitter, State};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::process::Command;
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LocalSession {
    pub id: String,
    pub shell: String,
    pub pid: u32,
}

/// Spawn a local shell process.
#[tauri::command]
pub async fn local_shell_spawn(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    shell: Option<String>,
) -> AppResult<LocalSession> {
    let session_id = uuid::Uuid::new_v4().to_string();

    #[cfg(target_os = "windows")]
    let (cmd, args): (String, Vec<String>) = {
        let shell = shell.unwrap_or_else(|| {
            if std::path::Path::new("C:\\Windows\\System32\\WindowsPowerShell\\v1.0\\powershell.exe").exists() {
                "powershell".to_string()
            } else {
                "cmd".to_string()
            }
        });
        match shell.as_str() {
            "powershell" | "pwsh" => ("powershell.exe".to_string(), vec!["-NoLogo".to_string()]),
            "cmd" => ("cmd.exe".to_string(), vec![]),
            "git-bash" => ("C:\\Program Files\\Git\\bin\\bash.exe".to_string(), vec!["--login".to_string()]),
            "wsl" => ("wsl.exe".to_string(), vec![]),
            _ => (shell, vec![]),
        }
    };

    #[cfg(not(target_os = "windows"))]
    let (cmd, args): (String, Vec<String>) = {
        let shell = shell.unwrap_or_else(|| {
            std::env::var("SHELL").unwrap_or_else(|_| "/bin/bash".to_string())
        });
        (shell, vec![])
    };

    let mut cmd_builder = Command::new(&cmd);
    cmd_builder
        .args(&args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true);

    #[cfg(target_os = "windows")]
    {
        cmd_builder.creation_flags(0x08000000); // CREATE_NO_WINDOW
    }

    let mut child = cmd_builder
        .spawn()
        .map_err(|e| crate::errors::AppError::Internal(format!("spawn {cmd}: {e}")))?;

    let pid = child.id().unwrap_or(0);

    let mut stdout = child.stdout.take().ok_or_else(|| crate::errors::AppError::Internal("failed to open stdout".into()))?;
    let mut stderr = child.stderr.take().ok_or_else(|| crate::errors::AppError::Internal("failed to open stderr".into()))?;

    // Spawn reader task for stdout
    let app_stdout = app.clone();
    let session_id_stdout = session_id.clone();
    tokio::spawn(async move {
        let mut buf = vec![0u8; 1024];
        loop {
            match stdout.read(&mut buf).await {
                Ok(0) => break, // EOF
                Ok(n) => {
                    let text = String::from_utf8_lossy(&buf[..n]).into_owned();
                    let _ = app_stdout.emit(&format!("ssh:output:{}", session_id_stdout), serde_json::json!({
                        "sessionId": session_id_stdout,
                        "data": text,
                    }));
                }
                Err(_) => break,
            }
        }
    });

    // Spawn reader task for stderr
    let app_stderr = app.clone();
    let session_id_stderr = session_id.clone();
    tokio::spawn(async move {
        let mut buf = vec![0u8; 1024];
        loop {
            match stderr.read(&mut buf).await {
                Ok(0) => break, // EOF
                Ok(n) => {
                    let text = String::from_utf8_lossy(&buf[..n]).into_owned();
                    let _ = app_stderr.emit(&format!("ssh:output:{}", session_id_stderr), serde_json::json!({
                        "sessionId": session_id_stderr,
                        "data": text,
                    }));
                }
                Err(_) => break,
            }
        }
    });

    // Store the child process handle.
    state.local_sessions.insert(session_id.clone(), tokio::sync::Mutex::new(child));

    // Spawn status monitoring task
    let app_status = app.clone();
    let session_id_status = session_id.clone();
    let local_sessions_status = Arc::clone(&state.local_sessions);
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(std::time::Duration::from_millis(200)).await;
            
            let mut exited = false;
            if let Some(entry) = local_sessions_status.get(&session_id_status) {
                if let Ok(mut child) = entry.value().try_lock() {
                    match child.try_wait() {
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
                let _ = app_status.emit(&format!("ssh:status:{}", session_id_status), serde_json::json!({
                    "sessionId": session_id_status,
                    "status": "disconnected",
                    "message": null,
                }));
                local_sessions_status.remove(&session_id_status);
                break;
            }
        }
    });

    // Emit connected status immediately
    let _ = app.emit(&format!("ssh:status:{}", session_id), serde_json::json!({
        "sessionId": session_id,
        "status": "connected",
        "message": null,
    }));

    Ok(LocalSession {
        id: session_id,
        shell: cmd,
        pid,
    })
}

/// Send data to a local shell.
#[tauri::command]
pub async fn local_shell_send(
    state: State<'_, AppState>,
    session_id: String,
    data: String,
) -> AppResult<()> {
    let entry = state.local_sessions.get(&session_id)
        .ok_or_else(|| crate::errors::AppError::NotFound("session not found".into()))?;
    let mut child = entry.value().lock().await;
    if let Some(ref mut stdin) = child.stdin {
        stdin.write_all(data.as_bytes()).await
            .map_err(|e| crate::errors::AppError::Internal(format!("write: {e}")))?;
        stdin.flush().await
            .map_err(|e| crate::errors::AppError::Internal(format!("flush: {e}")))?;
    }
    Ok(())
}

/// Read output from a local shell (non-blocking, returns available data).
#[tauri::command]
pub async fn local_shell_read(
    state: State<'_, AppState>,
    session_id: String,
) -> AppResult<String> {
    let entry = state.local_sessions.get(&session_id)
        .ok_or_else(|| crate::errors::AppError::NotFound("session not found".into()))?;
    let mut child = entry.value().lock().await;
    let mut output = String::new();
    if let Some(ref mut stdout) = child.stdout {
        let mut buf = vec![0u8; 4096];
        match tokio::time::timeout(
            std::time::Duration::from_millis(100),
            stdout.read(&mut buf),
        ).await {
            Ok(Ok(n)) if n > 0 => { output.push_str(&String::from_utf8_lossy(&buf[..n])); }
            _ => {}
        }
    }
    Ok(output)
}

/// Kill a local shell session.
#[tauri::command]
pub async fn local_shell_kill(
    state: State<'_, AppState>,
    session_id: String,
) -> AppResult<()> {
    if let Some((_, child)) = state.local_sessions.remove(&session_id) {
        let mut child = child.lock().await;
        child.kill().await.ok();
    }
    Ok(())
}

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
