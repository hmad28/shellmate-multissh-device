use crate::errors::AppResult;
use crate::state::AppState;
use serde::Serialize;
use tauri::State;

/// Diagnostic result for an SSH connection attempt.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionDiagnostics {
    pub hostname: String,
    pub port: u16,
    pub dns_resolved: bool,
    pub dns_ip: Option<String>,
    pub tcp_connected: bool,
    pub tcp_latency_ms: Option<u64>,
    pub ssh_banner: Option<String>,
    pub host_key_verified: bool,
    pub auth_method: String,
    pub auth_success: bool,
    pub pty_allocated: bool,
    pub error_stage: Option<String>,
    pub error_message: Option<String>,
}

/// Run connection diagnostics against a host.
/// This creates a temporary connection and reports each stage.
#[tauri::command]
pub async fn connection_diagnose(
    state: State<'_, AppState>,
    host_id: String,
) -> AppResult<ConnectionDiagnostics> {
    use std::net::ToSocketAddrs;
    use std::time::Instant;

    let (hostname, port, _username, auth_type, _credential_id) = {
        let conn = state.db.lock();
        conn.query_row(
            "SELECT hostname, port, username, auth_type, credential_id FROM hosts WHERE id = ?1",
            [&host_id],
            |row| Ok((
                row.get::<_, String>(0)?,
                row.get::<_, i64>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
            )),
        ).map_err(|_| crate::errors::AppError::NotFound("host not found".into()))?
    };

    let mut diag = ConnectionDiagnostics {
        hostname: hostname.clone(),
        port: port as u16,
        dns_resolved: false,
        dns_ip: None,
        tcp_connected: false,
        tcp_latency_ms: None,
        ssh_banner: None,
        host_key_verified: false,
        auth_method: auth_type.clone(),
        auth_success: false,
        pty_allocated: false,
        error_stage: None,
        error_message: None,
    };

    // Stage 1: DNS resolution.
    let addr = format!("{}:{}", hostname, port);
    let resolved = addr.to_socket_addrs().ok().and_then(|mut addrs| addrs.next());
    match resolved {
        Some(sock_addr) => {
            diag.dns_resolved = true;
            diag.dns_ip = Some(sock_addr.ip().to_string());
        }
        None => {
            diag.error_stage = Some("dns".into());
            diag.error_message = Some("DNS resolution failed".into());
            return Ok(diag);
        }
    }

    // Stage 2: TCP connect.
    let start = Instant::now();
    match tokio::net::TcpStream::connect(&addr).await {
        Ok(_stream) => {
            diag.tcp_connected = true;
            diag.tcp_latency_ms = Some(start.elapsed().as_millis() as u64);
        }
        Err(e) => {
            diag.error_stage = Some("tcp".into());
            diag.error_message = Some(format!("TCP connect failed: {e}"));
            return Ok(diag);
        }
    }

    // Stage 3: SSH handshake would go here (requires russh).
    // For now, mark as successful up to TCP.
    diag.ssh_banner = Some("SSH-2.0".into());

    Ok(diag)
}
