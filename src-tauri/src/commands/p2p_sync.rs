use crate::errors::{AppError, AppResult};
use crate::state::{AppState, LocalSessionState};
use crate::vault::Vault;
use dashmap::DashMap;
use aes_gcm::{aead::Aead, Aes256Gcm, KeyInit, Nonce};
use argon2::Argon2;
use base64::Engine;
use log::warn;
use parking_lot::Mutex;
use rand::Rng;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tauri::{AppHandle, Emitter, State};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::sync::oneshot;

const MAX_PIN_ATTEMPTS: usize = 10;
const RATE_LIMIT_WINDOW_SECS: u64 = 60;
const DEFAULT_P2P_PORT: u16 = 43177;

pub struct SyncServerState {
    inner: Mutex<SyncServerInner>,
}

struct SyncServerInner {
    shutdown_tx: Option<oneshot::Sender<()>>,
    pin: Option<String>,
    pairing_code: Option<String>,
    session_key: [u8; 32],
    is_running: bool,
    failed_attempts: HashMap<std::net::IpAddr, (u32, Instant)>,
}

impl SyncServerState {
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(SyncServerInner {
                shutdown_tx: None,
                pin: None,
                pairing_code: None,
                session_key: generate_session_key(),
                is_running: false,
                failed_attempts: HashMap::new(),
            }),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct SyncPayload {
    hosts: Vec<HostExport>,
    credentials: Vec<CredentialExport>,
    groups: Vec<GroupExport>,
    snippets: Vec<SnippetExport>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct PairingEndpoint {
    label: String,
    host: String,
    port: u16,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct PairingCode {
    v: u8,
    host: String,
    port: u16,
    pin: String,
    #[serde(default)]
    endpoints: Vec<PairingEndpoint>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PullRequest {
    device_id: String,
    device_name: String,
    pin: Option<String>,
    token: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PullResponse {
    device_id: String,
    token: String,
    payload: String,
    message: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TerminalRequest {
    device_id: String,
    device_name: String,
    token: String,
    session_id: Option<String>,
    shell: Option<String>,
    data: Option<String>,
    cols: Option<u16>,
    rows: Option<u16>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PairedDevice {
    id: String,
    device_name: String,
    bound_ip: String,
    paired_at: String,
    last_seen_at: String,
    revoked_at: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct HostExport {
    id: String,
    label: String,
    hostname: String,
    port: i64,
    username: String,
    auth_type: String,
    credential_id: String,
    group_id: Option<String>,
    tags: Option<String>,
    notes: Option<String>,
    created_at: String,
    updated_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct CredentialExport {
    id: String,
    #[serde(rename = "type")]
    cred_type: String,
    encrypted_data: Vec<u8>,
    nonce: Vec<u8>,
    created_at: String,
    updated_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct GroupExport {
    id: String,
    name: String,
    color: Option<String>,
    parent_id: Option<String>,
    sort_order: i64,
}

#[derive(Debug, Serialize, Deserialize)]
struct SnippetExport {
    id: String,
    title: String,
    command: String,
    description: Option<String>,
    tags: Option<String>,
    created_at: String,
    updated_at: String,
}

#[derive(Debug, Deserialize)]
struct SyncReceiveRequest {
    pin: String,
    session_key_encrypted: Vec<u8>,
    payload: Vec<u8>,
}

fn generate_pin() -> String {
    let mut rng = rand::thread_rng();
    format!("{:06}", rng.gen_range(0..1_000_000))
}

fn generate_token() -> String {
    let mut bytes = [0u8; 32];
    rand::thread_rng().fill(&mut bytes);
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(bytes)
}

fn token_hash(token: &str) -> String {
    use sha2::{Digest, Sha256};
    hex::encode(Sha256::digest(token.as_bytes()))
}

fn now_rfc3339() -> String {
    chrono::Utc::now().to_rfc3339()
}

fn local_lan_ip() -> String {
    route_local_ip("8.8.8.8:80")
        .unwrap_or_else(|_| "127.0.0.1".to_string())
}

fn route_local_ip(target: &str) -> std::io::Result<String> {
    std::net::UdpSocket::bind("0.0.0.0:0").and_then(|socket| {
        let _ = socket.connect(target);
        socket.local_addr().map(|addr| addr.ip().to_string())
    })
}

fn add_pairing_endpoint(
    endpoints: &mut Vec<PairingEndpoint>,
    label: &str,
    host: String,
    port: u16,
) {
    if host == "0.0.0.0" || endpoints.iter().any(|endpoint| endpoint.host == host) {
        return;
    }

    endpoints.push(PairingEndpoint {
        label: label.to_string(),
        host,
        port,
    });
}

fn is_tailscale_ip(host: &str) -> bool {
    let mut parts = host.split('.').filter_map(|part| part.parse::<u8>().ok());
    matches!((parts.next(), parts.next()), (Some(100), Some(second)) if (64..=127).contains(&second))
}

fn tailscale_cli_ip() -> Option<String> {
    let output = std::process::Command::new("tailscale")
        .args(["ip", "-4"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }

    String::from_utf8(output.stdout)
        .ok()?
        .lines()
        .map(str::trim)
        .find(|host| is_tailscale_ip(host))
        .map(str::to_string)
}

fn build_pairing_endpoints(port: u16) -> Vec<PairingEndpoint> {
    let mut endpoints = Vec::new();

    if let Some(host) = tailscale_cli_ip() {
        add_pairing_endpoint(&mut endpoints, "Tailscale/VPN", host, port);
    }
    if let Ok(host) = route_local_ip("100.100.100.100:53") {
        if is_tailscale_ip(&host) {
            add_pairing_endpoint(&mut endpoints, "Tailscale/VPN", host, port);
        }
    }
    add_pairing_endpoint(&mut endpoints, "LAN", local_lan_ip(), port);
    add_pairing_endpoint(&mut endpoints, "ADB reverse", "127.0.0.1".to_string(), port);

    endpoints
}

fn endpoints_from_pairing_code(code: &PairingCode) -> Vec<PairingEndpoint> {
    if !code.endpoints.is_empty() {
        return code.endpoints.clone();
    }

    vec![PairingEndpoint {
        label: "Desktop".to_string(),
        host: code.host.clone(),
        port: code.port,
    }]
}

async fn request_desktop_pull(
    client: &reqwest::Client,
    endpoints: &[PairingEndpoint],
    pull: &PullRequest,
    context: &str,
) -> AppResult<(PullResponse, String)> {
    let mut failures = Vec::new();

    for endpoint in endpoints {
        let url = format!("http://{}:{}/pull", endpoint.host, endpoint.port);
        let endpoint_name = format!("{} {}:{}", endpoint.label, endpoint.host, endpoint.port);
        let response = match client.post(&url).json(pull).send().await {
            Ok(response) => response,
            Err(error) => {
                failures.push(format!("{endpoint_name} -> {error}"));
                continue;
            }
        };

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            failures.push(format!("{endpoint_name} -> HTTP {status}: {body}"));
            continue;
        }

        let pull_response: PullResponse = response.json().await.map_err(|error| {
            AppError::Internal(format!("invalid {context} desktop response: {error}"))
        })?;
        return Ok((pull_response, format!("{}:{}", endpoint.host, endpoint.port)));
    }

    Err(AppError::Internal(format!(
        "{context} failed: could not reach any desktop endpoint ({}). For access outside the same Wi-Fi, connect both devices through a reachable tunnel/VPN such as Tailscale, WireGuard, or Cloudflare Tunnel; a private laptop IP cannot be reached directly from arbitrary internet.",
        failures.join("; ")
    )))
}

fn encode_pairing_code(code: &PairingCode) -> AppResult<String> {
    let json = serde_json::to_vec(code)
        .map_err(|e| AppError::Internal(format!("pairing code encode failed: {e}")))?;
    Ok(base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(json))
}

fn decode_pairing_code(input: &str) -> AppResult<PairingCode> {
    let trimmed = input.trim();
    let bytes = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(trimmed)
        .map_err(|_| AppError::InvalidInput("invalid pairing code".into()))?;
    serde_json::from_slice(&bytes).map_err(|_| AppError::InvalidInput("invalid pairing code".into()))
}

/// Generate a random 32-byte session key for P2P transfer encryption.
/// The PIN is NOT used as encryption key — it's only for authentication.
fn generate_session_key() -> [u8; 32] {
    let mut key = [0u8; 32];
    rand::thread_rng().fill(&mut key);
    key
}

/// Derive an encryption key from a session key using HKDF.
fn derive_encryption_key(session_key: &[u8; 32]) -> [u8; 32] {
    use hkdf::Hkdf;
    use sha2::Sha256;
    let hk = Hkdf::<Sha256>::new(Some(b"shellmate-p2p-v1"), session_key);
    let mut key = [0u8; 32];
    hk.expand(b"p2p-transfer-key", &mut key)
        .expect("HKDF expand failed");
    key
}

fn encrypt_payload(key: &[u8; 32], plaintext: &[u8]) -> AppResult<Vec<u8>> {
    let cipher = Aes256Gcm::new_from_slice(key)
        .map_err(|e| AppError::Internal(format!("cipher init failed: {e}")))?;
    let mut nonce_bytes = [0u8; 12];
    rand::thread_rng().fill(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);
    let ciphertext = cipher
        .encrypt(nonce, plaintext)
        .map_err(|e| AppError::Internal(format!("encryption failed: {e}")))?;
    let mut result = nonce_bytes.to_vec();
    result.extend(ciphertext);
    Ok(result)
}

fn decrypt_payload(key: &[u8; 32], data: &[u8]) -> AppResult<Vec<u8>> {
    if data.len() < 12 {
        return Err(AppError::InvalidInput("encrypted payload too short".into()));
    }
    let cipher = Aes256Gcm::new_from_slice(key)
        .map_err(|e| AppError::Internal(format!("cipher init failed: {e}")))?;
    let nonce = Nonce::from_slice(&data[..12]);
    cipher
        .decrypt(nonce, &data[12..])
        .map_err(|e| AppError::Internal(format!("decryption failed: {e}")))
}

fn export_sync_payload(db: &Arc<Mutex<Connection>>) -> AppResult<SyncPayload> {
    let conn = db.lock();

    let hosts: Vec<HostExport> = {
        let mut stmt = conn.prepare(
            "SELECT id, label, hostname, port, username, auth_type, credential_id, group_id, tags, notes, created_at, updated_at
             FROM hosts",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(HostExport {
                id: row.get(0)?,
                label: row.get(1)?,
                hostname: row.get(2)?,
                port: row.get(3)?,
                username: row.get(4)?,
                auth_type: row.get(5)?,
                credential_id: row.get(6)?,
                group_id: row.get(7)?,
                tags: row.get(8)?,
                notes: row.get(9)?,
                created_at: row.get(10)?,
                updated_at: row.get(11)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
        rows
    };

    let credentials: Vec<CredentialExport> = {
        let mut stmt = conn.prepare(
            "SELECT id, type, encrypted_data, nonce, created_at, updated_at FROM credentials",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(CredentialExport {
                id: row.get(0)?,
                cred_type: row.get(1)?,
                encrypted_data: row.get(2)?,
                nonce: row.get(3)?,
                created_at: row.get(4)?,
                updated_at: row.get(5)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
        rows
    };

    let groups: Vec<GroupExport> = {
        let mut stmt = conn.prepare("SELECT id, name, color, parent_id, sort_order FROM groups")?;
        let rows = stmt.query_map([], |row| {
            Ok(GroupExport {
                id: row.get(0)?,
                name: row.get(1)?,
                color: row.get(2)?,
                parent_id: row.get(3)?,
                sort_order: row.get(4)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
        rows
    };

    let snippets: Vec<SnippetExport> = {
        let mut stmt = conn.prepare(
            "SELECT id, title, command, description, tags, created_at, updated_at FROM snippets",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(SnippetExport {
                id: row.get(0)?,
                title: row.get(1)?,
                command: row.get(2)?,
                description: row.get(3)?,
                tags: row.get(4)?,
                created_at: row.get(5)?,
                updated_at: row.get(6)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
        rows
    };

    Ok(SyncPayload {
        hosts,
        credentials,
        groups,
        snippets,
    })
}

fn encode_sync_payload(payload: &SyncPayload) -> AppResult<String> {
    let json = serde_json::to_vec(payload)
        .map_err(|e| AppError::Internal(format!("serialization failed: {e}")))?;
    Ok(base64::engine::general_purpose::STANDARD.encode(json))
}

fn parse_http_request(request: &str) -> (String, String) {
    let first_line = request.lines().next().unwrap_or("");
    let parts: Vec<&str> = first_line.split_whitespace().collect();
    let method = parts.first().unwrap_or(&"").to_string();
    let path = parts.get(1).unwrap_or(&"/").to_string();
    (method, path)
}

async fn handle_sync_receive(
    stream: &mut tokio::net::TcpStream,
    raw_request: &[u8],
    expected_pin: &str,
    db: &Arc<Mutex<Connection>>,
    _vault: &Arc<Vault>,
    app: &AppHandle,
    peer_addr: Option<std::net::SocketAddr>,
    server_state: &SyncServerState,
) {
    let request_str = String::from_utf8_lossy(raw_request);
    let body_start = request_str.find("\r\n\r\n").unwrap_or(0) + 4;
    let body = &raw_request[body_start..];

    let req: SyncReceiveRequest = match serde_json::from_slice(body) {
        Ok(r) => r,
        Err(_) => {
            let response = "HTTP/1.1 400 Bad Request\r\nContent-Length: 0\r\n\r\n";
            let _ = stream.write_all(response.as_bytes()).await;
            return;
        }
    };

    if req.pin != expected_pin {
        // Track failed attempt with rate limiting.
        let rate_limited = if let Some(addr) = peer_addr {
            let mut inner = server_state.inner.lock();
            let now = Instant::now();
            let entry = inner.failed_attempts.entry(addr.ip()).or_insert((0, now));

            if now.duration_since(entry.1).as_secs() > RATE_LIMIT_WINDOW_SECS {
                *entry = (1, now);
                false
            } else {
                entry.0 += 1;
                entry.1 = now;
                if entry.0 >= MAX_PIN_ATTEMPTS as u32 {
                    warn!("Rate limit exceeded for {}", addr.ip());
                    true
                } else {
                    false
                }
            }
        } else {
            false
        };
        // Lock is dropped here before any .await.

        if rate_limited {
            let response =
                "HTTP/1.1 429 Too Many Requests\r\nRetry-After: 60\r\nContent-Length: 0\r\n\r\n";
            let _ = stream.write_all(response.as_bytes()).await;
        } else {
            let response = "HTTP/1.1 401 Unauthorized\r\nContent-Length: 0\r\n\r\n";
            let _ = stream.write_all(response.as_bytes()).await;
        }
        return;
    }

    // Decrypt the session key using PIN-derived key.
    let pin_salt = b"shellmate-p2p-pin-salt-v1";
    let pin_key = {
        let mut key = [0u8; 32];
        if Argon2::default()
            .hash_password_into(req.pin.as_bytes(), pin_salt, &mut key)
            .is_err()
        {
            let response = "HTTP/1.1 500 Internal Server Error\r\nContent-Length: 0\r\n\r\n";
            let _ = stream.write_all(response.as_bytes()).await;
            return;
        }
        key
    };
    let session_key = match decrypt_payload(&pin_key, &req.session_key_encrypted) {
        Ok(d) if d.len() == 32 => {
            let mut k = [0u8; 32];
            k.copy_from_slice(&d);
            k
        }
        _ => {
            let response = "HTTP/1.1 400 Bad Request\r\nContent-Length: 0\r\n\r\n";
            let _ = stream.write_all(response.as_bytes()).await;
            return;
        }
    };

    // Use session key for payload decryption.
    let enc_key = derive_encryption_key(&session_key);
    let decrypted = match decrypt_payload(&enc_key, &req.payload) {
        Ok(d) => d,
        Err(_) => {
            let response = "HTTP/1.1 400 Bad Request\r\nContent-Length: 0\r\n\r\n";
            let _ = stream.write_all(response.as_bytes()).await;
            return;
        }
    };

    let payload: SyncPayload = match serde_json::from_slice(&decrypted) {
        Ok(p) => p,
        Err(_) => {
            let response = "HTTP/1.1 400 Bad Request\r\nContent-Length: 0\r\n\r\n";
            let _ = stream.write_all(response.as_bytes()).await;
            return;
        }
    };

    let result = merge_payload(db, &payload);

    let (status, body) = match result {
        Ok(msg) => {
            let _ = app.emit("p2p:sync-complete", &msg);
            (
                "200 OK",
                serde_json::json!({"success": true, "message": msg}),
            )
        }
        Err(e) => {
            let msg = format!("Merge failed: {e}");
            (
                "500 Internal Server Error",
                serde_json::json!({"success": false, "message": msg}),
            )
        }
    };

    let body_str = body.to_string();
    let response = format!(
        "HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
        body_str.len(),
        body_str
    );
    let _ = stream.write_all(response.as_bytes()).await;
}

async fn write_json_response(stream: &mut tokio::net::TcpStream, status: &str, body: serde_json::Value) {
    let body_str = body.to_string();
    let response = format!(
        "HTTP/1.1 {status}\r\nContent-Type: application/json\r\nAccess-Control-Allow-Origin: *\r\nContent-Length: {}\r\n\r\n{}",
        body_str.len(),
        body_str
    );
    let _ = stream.write_all(response.as_bytes()).await;
}

async fn handle_pair_pull(
    stream: &mut tokio::net::TcpStream,
    raw_request: &[u8],
    expected_pin: &str,
    db: &Arc<Mutex<Connection>>,
    peer_addr: Option<std::net::SocketAddr>,
) {
    let request_str = String::from_utf8_lossy(raw_request);
    let body_start = request_str.find("\r\n\r\n").map(|i| i + 4).unwrap_or(raw_request.len());
    let body = &raw_request[body_start..];

    let req: PullRequest = match serde_json::from_slice(body) {
        Ok(req) => req,
        Err(_) => {
            write_json_response(
                stream,
                "400 Bad Request",
                serde_json::json!({"success": false, "message": "invalid pull request"}),
            )
            .await;
            return;
        }
    };

    let Some(peer_ip) = peer_addr.map(|addr| addr.ip().to_string()) else {
        write_json_response(
            stream,
            "400 Bad Request",
            serde_json::json!({"success": false, "message": "missing peer address"}),
        )
        .await;
        return;
    };

    let token = req.token.unwrap_or_default();
    let token_hash_value = token_hash(&token);
    let mut token_to_return = token.clone();

    let auth_result: AppResult<()> = {
        let conn = db.lock();
        if !token.is_empty() {
            let row = conn.query_row(
                "SELECT device_name, bound_ip, revoked_at FROM paired_devices WHERE id = ?1 AND token_hash = ?2",
                rusqlite::params![req.device_id, token_hash_value],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, Option<String>>(2)?,
                    ))
                },
            );

            match row {
                Ok((device_name, bound_ip, revoked_at)) => {
                    if revoked_at.is_some() {
                        Err(AppError::InvalidInput("device token revoked".into()))
                    } else if device_name != req.device_name || bound_ip != peer_ip {
                        Err(AppError::InvalidInput(
                            "device identity changed; pair again from desktop".into(),
                        ))
                    } else {
                        if let Err(err) = conn.execute(
                            "UPDATE paired_devices SET last_seen_at = ?1 WHERE id = ?2",
                            rusqlite::params![now_rfc3339(), req.device_id],
                        ) {
                            Err(AppError::Database(err))
                        } else {
                            Ok(())
                        }
                    }
                }
                Err(_) => Err(AppError::InvalidInput("unknown device token".into())),
            }
        } else if req.pin.as_deref() == Some(expected_pin) {
            token_to_return = generate_token();
            let now = now_rfc3339();
            if let Err(err) = conn.execute(
                "INSERT INTO paired_devices (id, device_name, token_hash, bound_ip, revoked_at, paired_at, last_seen_at)
                 VALUES (?1, ?2, ?3, ?4, NULL, ?5, ?5)
                 ON CONFLICT(id) DO UPDATE SET
                   device_name = excluded.device_name,
                   token_hash = excluded.token_hash,
                   bound_ip = excluded.bound_ip,
                   revoked_at = NULL,
                   paired_at = excluded.paired_at,
                   last_seen_at = excluded.last_seen_at",
                rusqlite::params![
                    req.device_id,
                    req.device_name,
                    token_hash(&token_to_return),
                    peer_ip,
                    now
                ],
            ) {
                Err(AppError::Database(err))
            } else {
                Ok(())
            }
        } else {
            Err(AppError::InvalidInput("invalid pairing code".into()))
        }
    };

    if let Err(err) = auth_result {
        write_json_response(
            stream,
            "401 Unauthorized",
            serde_json::json!({"success": false, "message": err.to_string()}),
        )
        .await;
        return;
    }

    let payload = match export_sync_payload(db).and_then(|payload| encode_sync_payload(&payload)) {
        Ok(payload) => payload,
        Err(err) => {
            write_json_response(
                stream,
                "500 Internal Server Error",
                serde_json::json!({"success": false, "message": err.to_string()}),
            )
            .await;
            return;
        }
    };

    let response = PullResponse {
        device_id: req.device_id,
        token: token_to_return,
        payload,
        message: "Device paired and synced".into(),
    };

    write_json_response(stream, "200 OK", serde_json::json!(response)).await;
}

fn request_body(raw_request: &[u8]) -> &[u8] {
    let request_str = String::from_utf8_lossy(raw_request);
    let body_start = request_str
        .find("\r\n\r\n")
        .map(|i| i + 4)
        .unwrap_or(raw_request.len());
    &raw_request[body_start..]
}

fn authorize_terminal_request(
    db: &Arc<Mutex<Connection>>,
    req: &TerminalRequest,
    peer_addr: Option<std::net::SocketAddr>,
) -> AppResult<()> {
    let peer_ip = peer_addr
        .map(|addr| addr.ip().to_string())
        .ok_or_else(|| AppError::InvalidInput("missing peer address".into()))?;
    let conn = db.lock();
    let row = conn.query_row(
        "SELECT device_name, bound_ip, revoked_at FROM paired_devices WHERE id = ?1 AND token_hash = ?2",
        rusqlite::params![req.device_id, token_hash(&req.token)],
        |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, Option<String>>(2)?,
            ))
        },
    );

    match row {
        Ok((device_name, bound_ip, revoked_at)) => {
            if revoked_at.is_some() {
                Err(AppError::InvalidInput("device token revoked".into()))
            } else if device_name != req.device_name || bound_ip != peer_ip {
                Err(AppError::InvalidInput(
                    "device identity changed; pair again from desktop".into(),
                ))
            } else {
                conn.execute(
                    "UPDATE paired_devices SET last_seen_at = ?1 WHERE id = ?2",
                    rusqlite::params![now_rfc3339(), req.device_id],
                )?;
                Ok(())
            }
        }
        Err(_) => Err(AppError::InvalidInput("unknown device token".into())),
    }
}

async fn parse_terminal_request(
    stream: &mut tokio::net::TcpStream,
    raw_request: &[u8],
) -> Option<TerminalRequest> {
    match serde_json::from_slice::<TerminalRequest>(request_body(raw_request)) {
        Ok(req) => Some(req),
        Err(_) => {
            write_json_response(
                stream,
                "400 Bad Request",
                serde_json::json!({"success": false, "message": "invalid terminal request"}),
            )
            .await;
            None
        }
    }
}

#[cfg(not(any(target_os = "android", target_os = "ios")))]
async fn handle_terminal_request(
    stream: &mut tokio::net::TcpStream,
    raw_request: &[u8],
    path: &str,
    app: &AppHandle,
    db: &Arc<Mutex<Connection>>,
    local_sessions: Arc<DashMap<String, tokio::sync::Mutex<LocalSessionState>>>,
    output_buffers: Arc<DashMap<String, tokio::sync::Mutex<String>>>,
    peer_addr: Option<std::net::SocketAddr>,
) {
    let Some(req) = parse_terminal_request(stream, raw_request).await else {
        return;
    };

    if let Err(err) = authorize_terminal_request(db, &req, peer_addr) {
        write_json_response(
            stream,
            "401 Unauthorized",
            serde_json::json!({"success": false, "message": err.to_string()}),
        )
        .await;
        return;
    }

    let result = match path {
        "/terminal/spawn" => {
            crate::commands::local_shell::spawn_local_shell(
                app.clone(),
                local_sessions,
                output_buffers,
                req.shell,
            )
            .await
            .map(|session| serde_json::json!({ "session": session }))
        }
        "/terminal/send" => {
            let session_id = req.session_id.unwrap_or_default();
            crate::commands::local_shell::send_local_shell(
                local_sessions,
                session_id,
                req.data.unwrap_or_default(),
            )
            .await
            .map(|_| serde_json::json!({ "ok": true }))
        }
        "/terminal/read" => {
            let session_id = req.session_id.unwrap_or_default();
            crate::commands::local_shell::read_local_shell(output_buffers, session_id)
                .await
                .map(|data| serde_json::json!({ "data": data }))
        }
        "/terminal/resize" => {
            let session_id = req.session_id.unwrap_or_default();
            crate::commands::local_shell::resize_local_shell(
                local_sessions,
                session_id,
                req.cols.unwrap_or(80),
                req.rows.unwrap_or(24),
            )
            .await
            .map(|_| serde_json::json!({ "ok": true }))
        }
        "/terminal/kill" => {
            let session_id = req.session_id.unwrap_or_default();
            crate::commands::local_shell::kill_local_shell(
                local_sessions,
                output_buffers,
                session_id,
            )
            .await
            .map(|_| serde_json::json!({ "ok": true }))
        }
        _ => Err(AppError::NotFound("terminal route".into())),
    };

    match result {
        Ok(body) => write_json_response(stream, "200 OK", body).await,
        Err(err) => {
            write_json_response(
                stream,
                "500 Internal Server Error",
                serde_json::json!({"success": false, "message": err.to_string()}),
            )
            .await
        }
    }
}

fn merge_payload(db: &Arc<Mutex<Connection>>, payload: &SyncPayload) -> AppResult<String> {
    let mut conn = db.lock();
    let tx = conn.transaction()?;
    let mut imported = 0u32;
    let mut skipped = 0u32;

    for group in &payload.groups {
        let exists: bool = tx
            .query_row(
                "SELECT COUNT(*) FROM groups WHERE id = ?1",
                [&group.id],
                |row| row.get::<_, i64>(0),
            )
            .map(|c| c > 0)
            .unwrap_or(false);

        if !exists {
            tx.execute(
                "INSERT INTO groups (id, name, color, parent_id, sort_order)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                rusqlite::params![
                    group.id,
                    group.name,
                    group.color,
                    group.parent_id,
                    group.sort_order
                ],
            )?;
            imported += 1;
        } else {
            skipped += 1;
        }
    }

    for cred in &payload.credentials {
        let exists: bool = tx
            .query_row(
                "SELECT COUNT(*) FROM credentials WHERE id = ?1",
                [&cred.id],
                |row| row.get::<_, i64>(0),
            )
            .map(|c| c > 0)
            .unwrap_or(false);

        if !exists {
            tx.execute(
                "INSERT INTO credentials (id, type, encrypted_data, nonce, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                rusqlite::params![
                    cred.id,
                    cred.cred_type,
                    cred.encrypted_data,
                    cred.nonce,
                    cred.created_at,
                    cred.updated_at
                ],
            )?;
            imported += 1;
        } else {
            skipped += 1;
        }
    }

    for host in &payload.hosts {
        let exists: bool = tx
            .query_row(
                "SELECT COUNT(*) FROM hosts WHERE id = ?1",
                [&host.id],
                |row| row.get::<_, i64>(0),
            )
            .map(|c| c > 0)
            .unwrap_or(false);

        if !exists {
            tx.execute(
                "INSERT INTO hosts (id, label, hostname, port, username, auth_type, credential_id, group_id, tags, notes, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
                rusqlite::params![
                    host.id,
                    host.label,
                    host.hostname,
                    host.port,
                    host.username,
                    host.auth_type,
                    host.credential_id,
                    host.group_id,
                    host.tags,
                    host.notes,
                    host.created_at,
                    host.updated_at
                ],
            )?;
            imported += 1;
        } else {
            skipped += 1;
        }
    }

    for snippet in &payload.snippets {
        let exists: bool = tx
            .query_row(
                "SELECT COUNT(*) FROM snippets WHERE id = ?1",
                [&snippet.id],
                |row| row.get::<_, i64>(0),
            )
            .map(|c| c > 0)
            .unwrap_or(false);

        if !exists {
            tx.execute(
                "INSERT INTO snippets (id, title, command, description, tags, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                rusqlite::params![
                    snippet.id,
                    snippet.title,
                    snippet.command,
                    snippet.description,
                    snippet.tags,
                    snippet.created_at,
                    snippet.updated_at
                ],
            )?;
            imported += 1;
        } else {
            skipped += 1;
        }
    }

    tx.commit()?;

    Ok(format!(
        "Imported {imported} items, skipped {skipped} duplicates"
    ))
}

async fn run_server(
    app: AppHandle,
    listener: TcpListener,
    port: u16,
    pin: String,
    pairing_code: String,
    db: Arc<Mutex<Connection>>,
    vault: Arc<Vault>,
    local_sessions: Arc<DashMap<String, tokio::sync::Mutex<LocalSessionState>>>,
    local_session_output: Arc<DashMap<String, tokio::sync::Mutex<String>>>,
    server_state: Arc<SyncServerState>,
    mut shutdown_rx: oneshot::Receiver<()>,
) {
    log::info!("P2P sync server listening on port {port}");

    let _ = app.emit("p2p:pairing-ready", &pairing_code);

    loop {
        tokio::select! {
            _ = &mut shutdown_rx => {
                log::info!("P2P sync server shutting down");
                break;
            }
            result = listener.accept() => {
                match result {
                    Ok((mut stream, _addr)) => {
                        let app = app.clone();
                        let db = db.clone();
                        let vault = vault.clone();
                        let local_sessions = Arc::clone(&local_sessions);
                        let local_session_output = Arc::clone(&local_session_output);
                        let pin = pin.clone();
                        let server_state_ref = server_state.clone();

                        tokio::spawn(async move {
                            let peer_addr = stream.peer_addr().ok();

                            // Rate limit check
                            if let Some(addr) = peer_addr {
                                let should_block = {
                                    let mut inner = server_state_ref.inner.lock();
                                    let now = Instant::now();
                                    let entry = inner.failed_attempts.entry(addr.ip()).or_insert((0, now));
                                    if now.duration_since(entry.1).as_secs() > RATE_LIMIT_WINDOW_SECS {
                                        *entry = (0, now);
                                    }
                                    entry.0 >= MAX_PIN_ATTEMPTS as u32
                                };
                                if should_block {
                                    let response = "HTTP/1.1 429 Too Many Requests\r\nContent-Length: 0\r\n\r\n";
                                    let _ = stream.write_all(response.as_bytes()).await;
                                    return;
                                }
                            }
                            let mut buf = vec![0u8; 65536];
                            let n = match stream.read(&mut buf).await {
                                Ok(n) if n > 0 => n,
                                _ => return,
                            };
                            buf.truncate(n);

                            let request = String::from_utf8_lossy(&buf);
                            let (method, path) = parse_http_request(&request);

                            match (method.as_str(), path.as_str()) {
                                ("GET", "/pair") => {
                                    // Return server info + session key encrypted with PIN.
                                    // PIN is NOT returned — client must know it from server display.
                                    let session_key_encrypted = {
                                        let inner = server_state_ref.inner.lock();
                                        let pin_salt = b"shellmate-p2p-pin-salt-v1";
                                        let mut pin_key = [0u8; 32];
                                        let _ = Argon2::default().hash_password_into(pin.as_bytes(), pin_salt, &mut pin_key);
                                        let enc_key = derive_encryption_key(&pin_key);
                                        encrypt_payload(&enc_key, &inner.session_key).ok()
                                    };
                                    let body = serde_json::json!({
                                        "port": port,
                                        "session_key_encrypted": session_key_encrypted,
                                    });
                                    let response = format!(
                                        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
                                        body.to_string().len(),
                                        body
                                    );
                                    let _ = stream.write_all(response.as_bytes()).await;
                                }
                                ("POST", "/pull") => {
                                    handle_pair_pull(
                                        &mut stream,
                                        &buf,
                                        &pin,
                                        &db,
                                        peer_addr,
                                    ).await;
                                }
                                ("POST", "/sync") => {
                                    handle_sync_receive(
                                        &mut stream,
                                        &buf,
                                        &pin,
                                        &db,
                                        &vault,
                                        &app,
                                        peer_addr,
                                        &server_state_ref,
                                    ).await;
                                }
                                #[cfg(not(any(target_os = "android", target_os = "ios")))]
                                ("POST", "/terminal/spawn")
                                | ("POST", "/terminal/send")
                                | ("POST", "/terminal/read")
                                | ("POST", "/terminal/resize")
                                | ("POST", "/terminal/kill") => {
                                    handle_terminal_request(
                                        &mut stream,
                                        &buf,
                                        path.as_str(),
                                        &app,
                                        &db,
                                        local_sessions,
                                        local_session_output,
                                        peer_addr,
                                    )
                                    .await;
                                }
                                _ => {
                                    let response = "HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\n\r\n";
                                    let _ = stream.write_all(response.as_bytes()).await;
                                }
                            }
                        });
                    }
                    Err(_) => {}
                }
            }
        }
    }

    let mut inner = server_state.inner.lock();
    inner.is_running = false;
    inner.pin = None;
    inner.pairing_code = None;
    inner.shutdown_tx = None;
}

#[tauri::command]
pub async fn p2p_start_sync_server(
    app: AppHandle,
    state: State<'_, Arc<SyncServerState>>,
    app_state: State<'_, AppState>,
) -> AppResult<String> {
    start_sync_server_internal(
        app,
        Arc::clone(&state),
        Arc::clone(&app_state.db),
        Arc::clone(&app_state.vault),
        Arc::clone(&app_state.local_sessions),
        Arc::clone(&app_state.local_session_output),
    )
    .await
}

pub async fn start_sync_server_internal(
    app: AppHandle,
    state: Arc<SyncServerState>,
    db: Arc<Mutex<Connection>>,
    vault: Arc<Vault>,
    local_sessions: Arc<DashMap<String, tokio::sync::Mutex<LocalSessionState>>>,
    local_session_output: Arc<DashMap<String, tokio::sync::Mutex<String>>>,
) -> AppResult<String> {
    {
        let inner = state.inner.lock();
        if inner.is_running {
            return inner.pairing_code.clone().ok_or_else(|| {
                AppError::Internal("sync server is running without a pairing code".into())
            });
        }
    }

    let (shutdown_tx, shutdown_rx) = oneshot::channel();
    let listener = match TcpListener::bind(("0.0.0.0", DEFAULT_P2P_PORT)).await {
        Ok(listener) => listener,
        Err(_) => TcpListener::bind("0.0.0.0:0")
            .await
            .map_err(|e| AppError::Internal(format!("failed to bind sync server: {e}")))?,
    };
    let port = listener
        .local_addr()
        .map_err(|e| AppError::Internal(format!("failed to read sync server port: {e}")))?
        .port();
    let pin = generate_pin();
    let endpoints = build_pairing_endpoints(port);
    let primary_endpoint = endpoints.first().cloned().unwrap_or_else(|| PairingEndpoint {
        label: "Desktop".to_string(),
        host: local_lan_ip(),
        port,
    });
    let pairing_code = encode_pairing_code(&PairingCode {
        v: 2,
        host: primary_endpoint.host,
        port: primary_endpoint.port,
        pin: pin.clone(),
        endpoints,
    })?;

    {
        let mut inner = state.inner.lock();
        inner.is_running = true;
        inner.pin = Some(pin.clone());
        inner.pairing_code = Some(pairing_code.clone());
        inner.shutdown_tx = Some(shutdown_tx);
    }

    let server_state = Arc::clone(&state);

    tokio::spawn(run_server(
        app,
        listener,
        port,
        pin,
        pairing_code.clone(),
        db,
        vault,
        local_sessions,
        local_session_output,
        server_state,
        shutdown_rx,
    ));

    Ok(pairing_code)
}

#[tauri::command]
pub async fn p2p_stop_sync_server(state: State<'_, Arc<SyncServerState>>) -> AppResult<()> {
    let mut inner = state.inner.lock();
    if let Some(tx) = inner.shutdown_tx.take() {
        let _ = tx.send(());
    }
    inner.is_running = false;
    inner.pin = None;
    inner.pairing_code = None;
    Ok(())
}

#[tauri::command]
pub async fn p2p_get_sync_status(
    state: State<'_, Arc<SyncServerState>>,
) -> AppResult<serde_json::Value> {
    let inner = state.inner.lock();
    Ok(serde_json::json!({
        "isRunning": inner.is_running,
        "hasPin": inner.pin.is_some(),
        "pairingCode": inner.pairing_code,
    }))
}

#[tauri::command]
pub async fn p2p_export_for_sync(app_state: State<'_, AppState>) -> AppResult<String> {
    encode_sync_payload(&export_sync_payload(&app_state.db)?)
}

#[tauri::command]
pub async fn p2p_list_paired_devices(
    app_state: State<'_, AppState>,
) -> AppResult<Vec<PairedDevice>> {
    let conn = app_state.db.lock();
    let mut stmt = conn.prepare(
        "SELECT id, device_name, bound_ip, paired_at, last_seen_at, revoked_at
         FROM paired_devices
         ORDER BY COALESCE(last_seen_at, paired_at) DESC",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(PairedDevice {
            id: row.get(0)?,
            device_name: row.get(1)?,
            bound_ip: row.get(2)?,
            paired_at: row.get(3)?,
            last_seen_at: row.get(4)?,
            revoked_at: row.get(5)?,
        })
    })?;
    rows.collect::<Result<Vec<_>, _>>().map_err(AppError::from)
}

#[tauri::command]
pub async fn p2p_revoke_paired_device(
    app_state: State<'_, AppState>,
    device_id: String,
) -> AppResult<()> {
    let conn = app_state.db.lock();
    let changed = conn.execute(
        "UPDATE paired_devices SET revoked_at = ?1 WHERE id = ?2 AND revoked_at IS NULL",
        rusqlite::params![now_rfc3339(), device_id],
    )?;

    if changed == 0 {
        return Err(AppError::InvalidInput(
            "paired device not found or already revoked".into(),
        ));
    }

    Ok(())
}

fn get_or_create_device_id(conn: &Connection) -> AppResult<String> {
    let existing = conn
        .query_row(
            "SELECT value FROM settings WHERE key = 'p2p.device_id'",
            [],
            |row| row.get::<_, String>(0),
        )
        .ok();
    if let Some(id) = existing {
        return Ok(id);
    }

    let id = uuid::Uuid::new_v4().to_string();
    conn.execute(
        "INSERT INTO settings (key, value) VALUES ('p2p.device_id', ?1)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        [&id],
    )?;
    Ok(id)
}

#[tauri::command]
pub async fn p2p_pair_with_desktop(
    app_state: State<'_, AppState>,
    pairing_code: String,
    device_name: Option<String>,
) -> AppResult<String> {
    let code = decode_pairing_code(&pairing_code)?;
    if code.v != 1 && code.v != 2 {
        return Err(AppError::InvalidInput("unsupported pairing code".into()));
    }

    let device_name = device_name
        .map(|name| name.trim().to_string())
        .filter(|name| !name.is_empty())
        .unwrap_or_else(|| "ShellMate Mobile".to_string());

    let device_id = {
        let conn = app_state.db.lock();
        let device_id = get_or_create_device_id(&conn)?;
        conn.execute(
            "INSERT INTO settings (key, value) VALUES ('p2p.device_name', ?1)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            [&device_name],
        )?;
        device_id
    };

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| AppError::Internal(format!("pairing client init failed: {e}")))?;
    let endpoints = endpoints_from_pairing_code(&code);
    let pull = PullRequest {
        device_id: device_id.clone(),
        device_name: device_name.clone(),
        pin: Some(code.pin.clone()),
        token: None,
    };

    let (pull_response, endpoint) =
        request_desktop_pull(&client, &endpoints, &pull, "desktop pairing").await?;

    let payload_bytes = base64::engine::general_purpose::STANDARD
        .decode(pull_response.payload.as_bytes())
        .map_err(|_| AppError::InvalidInput("invalid sync payload".into()))?;
    let payload: SyncPayload = serde_json::from_slice(&payload_bytes)
        .map_err(|e| AppError::InvalidInput(format!("invalid sync payload: {e}")))?;

    let message = merge_payload(&app_state.db, &payload)?;

    {
        let conn = app_state.db.lock();
        conn.execute(
            "INSERT INTO settings (key, value) VALUES ('p2p.desktop_endpoint', ?1)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            [&endpoint],
        )?;
        conn.execute(
            "INSERT INTO settings (key, value) VALUES ('p2p.desktop_pairing_code', ?1)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            [pairing_code],
        )?;
        conn.execute(
            "INSERT INTO settings (key, value) VALUES ('p2p.desktop_token', ?1)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            [pull_response.token],
        )?;
    }

    Ok(format!("{}; {}", pull_response.message, message))
}

#[tauri::command]
pub async fn p2p_sync_with_saved_desktop(app_state: State<'_, AppState>) -> AppResult<String> {
    let (pairing_code, token, device_id, device_name) = {
        let conn = app_state.db.lock();
        let get_setting = |key: &str| -> Option<String> {
            conn.query_row("SELECT value FROM settings WHERE key = ?1", [key], |row| {
                row.get::<_, String>(0)
            })
            .ok()
        };
        (
            get_setting("p2p.desktop_pairing_code"),
            get_setting("p2p.desktop_token"),
            get_setting("p2p.device_id"),
            get_setting("p2p.device_name").unwrap_or_else(|| "ShellMate Mobile".into()),
        )
    };

    let pairing_code =
        pairing_code.ok_or_else(|| AppError::InvalidInput("no paired desktop saved".into()))?;
    let token = token.ok_or_else(|| AppError::InvalidInput("no desktop token saved".into()))?;
    let device_id =
        device_id.ok_or_else(|| AppError::InvalidInput("no paired device id saved".into()))?;
    let code = decode_pairing_code(&pairing_code)?;
    if code.v != 1 && code.v != 2 {
        return Err(AppError::InvalidInput("unsupported pairing code".into()));
    }

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(8))
        .build()
        .map_err(|e| AppError::Internal(format!("sync client init failed: {e}")))?;
    let endpoints = endpoints_from_pairing_code(&code);
    let pull = PullRequest {
        device_id,
        device_name,
        pin: None,
        token: Some(token),
    };

    let (pull_response, endpoint) =
        request_desktop_pull(&client, &endpoints, &pull, "saved desktop sync").await?;
    {
        let conn = app_state.db.lock();
        conn.execute(
            "INSERT INTO settings (key, value) VALUES ('p2p.desktop_endpoint', ?1)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            [&endpoint],
        )?;
    }
    let payload_bytes = base64::engine::general_purpose::STANDARD
        .decode(pull_response.payload.as_bytes())
        .map_err(|_| AppError::InvalidInput("invalid sync payload".into()))?;
    let payload: SyncPayload = serde_json::from_slice(&payload_bytes)
        .map_err(|e| AppError::InvalidInput(format!("invalid sync payload: {e}")))?;
    merge_payload(&app_state.db, &payload)
}

pub async fn p2p_post_desktop_terminal(
    app_state: &AppState,
    path: &str,
    mut body: serde_json::Map<String, serde_json::Value>,
) -> AppResult<serde_json::Value> {
    let (pairing_code, token, device_id, device_name) = {
        let conn = app_state.db.lock();
        let get_setting = |key: &str| -> Option<String> {
            conn.query_row("SELECT value FROM settings WHERE key = ?1", [key], |row| {
                row.get::<_, String>(0)
            })
            .ok()
        };
        (
            get_setting("p2p.desktop_pairing_code"),
            get_setting("p2p.desktop_token"),
            get_setting("p2p.device_id"),
            get_setting("p2p.device_name").unwrap_or_else(|| "ShellMate Mobile".into()),
        )
    };

    let pairing_code =
        pairing_code.ok_or_else(|| AppError::InvalidInput("no paired desktop saved".into()))?;
    let token = token.ok_or_else(|| AppError::InvalidInput("no desktop token saved".into()))?;
    let device_id =
        device_id.ok_or_else(|| AppError::InvalidInput("no paired device id saved".into()))?;
    let code = decode_pairing_code(&pairing_code)?;
    let endpoints = endpoints_from_pairing_code(&code);

    body.insert("deviceId".into(), serde_json::json!(device_id));
    body.insert("deviceName".into(), serde_json::json!(device_name));
    body.insert("token".into(), serde_json::json!(token));

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(8))
        .build()
        .map_err(|e| AppError::Internal(format!("desktop terminal client init failed: {e}")))?;

    let mut failures = Vec::new();
    for endpoint in endpoints {
        let url = format!("http://{}:{}{}", endpoint.host, endpoint.port, path);
        match client.post(&url).json(&body).send().await {
            Ok(response) if response.status().is_success() => {
                return response.json().await.map_err(|error| {
                    AppError::Internal(format!("invalid desktop terminal response: {error}"))
                });
            }
            Ok(response) => {
                let status = response.status();
                let text = response.text().await.unwrap_or_default();
                failures.push(format!("{} {}:{} -> HTTP {status}: {text}", endpoint.label, endpoint.host, endpoint.port));
            }
            Err(error) => failures.push(format!(
                "{} {}:{} -> {error}",
                endpoint.label, endpoint.host, endpoint.port
            )),
        }
    }

    Err(AppError::Internal(format!(
        "desktop terminal request failed: could not reach any desktop endpoint ({})",
        failures.join("; ")
    )))
}
