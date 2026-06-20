use crate::errors::{AppError, AppResult};
use crate::state::{AppState, LocalSessionState};
use crate::vault::Vault;
use aes_gcm::{aead::Aead, Aes256Gcm, KeyInit, Nonce};
use argon2::Argon2;
use base64::Engine;
use dashmap::DashMap;
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

static REQWEST_CLIENT: std::sync::OnceLock<reqwest::Client> = std::sync::OnceLock::new();

fn get_reqwest_client() -> &'static reqwest::Client {
    REQWEST_CLIENT.get_or_init(|| {
        reqwest::Client::builder()
            .pool_max_idle_per_host(4)
            .pool_idle_timeout(std::time::Duration::from_secs(90))
            .tcp_keepalive(std::time::Duration::from_secs(15))
            .build()
            .expect("failed to build shared reqwest client")
    })
}

pub struct SyncServerState {
    inner: Mutex<SyncServerInner>,
}

struct SyncServerInner {
    shutdown_tx: Option<oneshot::Sender<()>>,
    pin: Option<String>,
    pairing_code: Option<String>,
    port: Option<u16>,
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
                port: None,
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
    #[serde(default = "default_encrypted_flag")]
    encrypted: bool,
}

fn default_encrypted_flag() -> bool {
    true
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
    #[serde(default)]
    wants_unlock: bool,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PullResponse {
    device_id: String,
    token: String,
    payload: String,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    session_key_encrypted: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    vault_key_export: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct VaultKeyExport {
    vault_key: String,
    vault_salt: String,
    verifier_ct: String,
    verifier_nonce: String,
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

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SshConnectRequest {
    device_id: String,
    device_name: String,
    token: String,
    host_id: String,
    session_id: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SshQuickConnectRequest {
    device_id: String,
    device_name: String,
    token: String,
    hostname: String,
    port: u16,
    username: String,
    label: Option<String>,
    auth: crate::commands::ssh::QuickConnectAuth,
    shell: Option<String>,
    session_id: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct DesktopAuthRequest {
    device_id: String,
    device_name: String,
    token: String,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct DesktopInputRequest {
    device_id: String,
    device_name: String,
    token: String,
    event: String, // "move", "click", "keydown", "keyup"
    x: Option<i32>,
    y: Option<i32>,
    button: Option<String>, // "left", "right", "middle"
    key: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct FileListRequest {
    device_id: String,
    device_name: String,
    token: String,
    path: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct FileDownloadRequest {
    device_id: String,
    device_name: String,
    token: String,
    path: String,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct FileDeleteRequest {
    device_id: String,
    device_name: String,
    token: String,
    path: String,
}

fn get_http_header(request: &str, header_name: &str) -> Option<String> {
    let lower_name = header_name.to_lowercase();
    for line in request.lines() {
        if let Some(pos) = line.find(':') {
            let key = line[..pos].trim().to_lowercase();
            if key == lower_name {
                return Some(line[pos + 1..].trim().to_string());
            }
        }
    }
    None
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
    #[serde(skip_serializing_if = "Option::is_none")]
    plaintext: Option<String>,
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
    route_local_ip("8.8.8.8:80").unwrap_or_else(|_| "127.0.0.1".to_string())
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
    #[cfg(target_os = "windows")]
    let cmd = {
        let paths = [
            "C:\\Program Files\\Tailscale\\tailscale.exe",
            "C:\\Program Files (x86)\\Tailscale\\tailscale.exe",
        ];
        paths.iter()
            .map(std::path::Path::new)
            .find(|p| p.exists())
            .map(|p| p.to_string_lossy().into_owned())
            .unwrap_or_else(|| {
                if let Ok(localappdata) = std::env::var("LOCALAPPDATA") {
                    let p = std::path::PathBuf::from(localappdata)
                        .join("Programs")
                        .join("Tailscale")
                        .join("tailscale.exe");
                    if p.exists() {
                        return p.to_string_lossy().into_owned();
                    }
                }
                "tailscale".to_string()
            })
    };
    #[cfg(not(target_os = "windows"))]
    let cmd = "tailscale";

    let output = std::process::Command::new(cmd)
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

fn tailscale_get_dns_name() -> Option<String> {
    #[cfg(target_os = "windows")]
    let cmd = {
        let paths = [
            "C:\\Program Files\\Tailscale\\tailscale.exe",
            "C:\\Program Files (x86)\\Tailscale\\tailscale.exe",
        ];
        paths.iter()
            .map(std::path::Path::new)
            .find(|p| p.exists())
            .map(|p| p.to_string_lossy().into_owned())
            .unwrap_or_else(|| {
                if let Ok(localappdata) = std::env::var("LOCALAPPDATA") {
                    let p = std::path::PathBuf::from(localappdata)
                        .join("Programs")
                        .join("Tailscale")
                        .join("tailscale.exe");
                    if p.exists() {
                        return p.to_string_lossy().into_owned();
                    }
                }
                "tailscale".to_string()
            })
    };
    #[cfg(not(target_os = "windows"))]
    let cmd = "tailscale";

    let output = std::process::Command::new(cmd)
        .args(["status", "--json"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }

    let status: serde_json::Value = serde_json::from_slice(&output.stdout).ok()?;
    let dns_name = status.get("Self")?.get("DNSName")?.as_str()?;
    Some(dns_name.trim_end_matches('.').to_string())
}

fn tailscale_start_serve(port: u16, funnel: bool) -> Result<(), std::io::Error> {
    #[cfg(target_os = "windows")]
    let cmd = {
        let paths = [
            "C:\\Program Files\\Tailscale\\tailscale.exe",
            "C:\\Program Files (x86)\\Tailscale\\tailscale.exe",
        ];
        paths.iter()
            .map(std::path::Path::new)
            .find(|p| p.exists())
            .map(|p| p.to_string_lossy().into_owned())
            .unwrap_or_else(|| {
                if let Ok(localappdata) = std::env::var("LOCALAPPDATA") {
                    let p = std::path::PathBuf::from(localappdata)
                        .join("Programs")
                        .join("Tailscale")
                        .join("tailscale.exe");
                    if p.exists() {
                        return p.to_string_lossy().into_owned();
                    }
                }
                "tailscale".to_string()
            })
    };
    #[cfg(not(target_os = "windows"))]
    let cmd = "tailscale";

    let mode = if funnel { "funnel" } else { "serve" };
    log::info!("Starting tailscale {} for port {}", mode, port);

    // Disable other mode first to prevent conflict
    let other_mode = if funnel { "serve" } else { "funnel" };
    let _ = std::process::Command::new(&cmd)
        .args([other_mode, &port.to_string(), "off"])
        .status();

    let status = std::process::Command::new(&cmd)
        .args([mode, "--bg", "--yes", &port.to_string()])
        .status()?;
    if !status.success() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("tailscale {mode} exited with error"),
        ));
    }
    Ok(())
}

fn tailscale_stop_serve(port: u16) {
    #[cfg(target_os = "windows")]
    let cmd = {
        let paths = [
            "C:\\Program Files\\Tailscale\\tailscale.exe",
            "C:\\Program Files (x86)\\Tailscale\\tailscale.exe",
        ];
        paths.iter()
            .map(std::path::Path::new)
            .find(|p| p.exists())
            .map(|p| p.to_string_lossy().into_owned())
            .unwrap_or_else(|| {
                if let Ok(localappdata) = std::env::var("LOCALAPPDATA") {
                    let p = std::path::PathBuf::from(localappdata)
                        .join("Programs")
                        .join("Tailscale")
                        .join("tailscale.exe");
                    if p.exists() {
                        return p.to_string_lossy().into_owned();
                    }
                }
                "tailscale".to_string()
            })
    };
    #[cfg(not(target_os = "windows"))]
    let cmd = "tailscale";

    log::info!("Stopping tailscale serve and funnel for port {}", port);
    let _ = std::process::Command::new(&cmd)
        .args(["serve", &port.to_string(), "off"])
        .status();
    let _ = std::process::Command::new(&cmd)
        .args(["funnel", &port.to_string(), "off"])
        .status();
}

fn build_pairing_endpoints(port: u16, tailscale_mode: &str) -> Vec<PairingEndpoint> {
    let mut endpoints = Vec::new();

    if tailscale_mode == "serve" || tailscale_mode == "funnel" {
        if let Some(dns_name) = tailscale_get_dns_name() {
            let label = if tailscale_mode == "funnel" {
                "Tailscale (Public)"
            } else {
                "Tailscale (Private)"
            };
            add_pairing_endpoint(&mut endpoints, label, dns_name, 443);
        }
    }

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

fn endpoint_from_host_port(value: &str) -> Option<PairingEndpoint> {
    let (host, port) = value.rsplit_once(':')?;
    let port = port.parse::<u16>().ok()?;
    if host.trim().is_empty() {
        return None;
    }
    Some(PairingEndpoint {
        label: "Last used".to_string(),
        host: host.to_string(),
        port,
    })
}

async fn request_desktop_pull(
    client: &reqwest::Client,
    endpoints: &[PairingEndpoint],
    pull: &PullRequest,
    context: &str,
) -> AppResult<(PullResponse, String)> {
    let mut failures = Vec::new();

    for endpoint in endpoints {
        let scheme = if endpoint.port == 443 { "https" } else { "http" };
        let url = format!("{}://{}:{}/pull", scheme, endpoint.host, endpoint.port);
        let endpoint_name = format!("{} {}://{}:{}", endpoint.label, scheme, endpoint.host, endpoint.port);
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
        return Ok((
            pull_response,
            format!("{}:{}", endpoint.host, endpoint.port),
        ));
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
    serde_json::from_slice(&bytes)
        .map_err(|_| AppError::InvalidInput("invalid pairing code".into()))
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

fn export_sync_payload(db: &Arc<Mutex<Connection>>, vault: &Arc<Vault>) -> AppResult<SyncPayload> {
    let conn = db.lock();

    let hosts: Vec<HostExport> = {
        let mut stmt = conn.prepare(
            "SELECT id, label, hostname, port, username, auth_type, credential_id, group_id, tags, notes, created_at, updated_at
             FROM hosts",
        )?;
        let rows = stmt
            .query_map([], |row| {
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

    let vault_unlocked = vault.is_unlocked();
    let credentials: Vec<CredentialExport> = {
        let mut stmt = conn.prepare(
            "SELECT id, type, encrypted_data, nonce, created_at, updated_at FROM credentials",
        )?;
        let rows = stmt
            .query_map([], |row| {
                let id: String = row.get(0)?;
                let cred_type: String = row.get(1)?;
                let encrypted_data: Vec<u8> = row.get(2)?;
                let nonce: Vec<u8> = row.get(3)?;
                let created_at: String = row.get(4)?;
                let updated_at: String = row.get(5)?;

                // Decrypt credential to plaintext if vault is unlocked so the
                // receiving device can re-encrypt with its own vault key.
                let plaintext = if vault_unlocked && nonce.len() == 12 {
                    let mut n = [0u8; 12];
                    n.copy_from_slice(&nonce);
                    let blob = crate::crypto::EncryptedBlob {
                        ciphertext: encrypted_data.clone(),
                        nonce: n,
                    };
                    vault
                        .decrypt(&blob)
                        .ok()
                        .and_then(|bytes| String::from_utf8(bytes).ok())
                } else {
                    None
                };

                Ok(CredentialExport {
                    id,
                    cred_type,
                    encrypted_data,
                    nonce,
                    created_at,
                    updated_at,
                    plaintext,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        rows
    };

    let groups: Vec<GroupExport> = {
        let mut stmt = conn.prepare("SELECT id, name, color, parent_id, sort_order FROM groups")?;
        let rows = stmt
            .query_map([], |row| {
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
        let rows = stmt
            .query_map([], |row| {
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
        encrypted: !vault_unlocked,
    })
}

fn encode_sync_payload(payload: &SyncPayload) -> AppResult<String> {
    let json = serde_json::to_vec(payload)
        .map_err(|e| AppError::Internal(format!("serialization failed: {e}")))?;
    Ok(base64::engine::general_purpose::STANDARD.encode(json))
}

fn serialize_sync_payload(payload: &SyncPayload) -> AppResult<Vec<u8>> {
    serde_json::to_vec(payload)
        .map_err(|e| AppError::Internal(format!("serialization failed: {e}")))
}

fn generate_pairing_code_for_port(port: u16, tailscale_mode: &str) -> AppResult<(String, String)> {
    let pin = generate_pin();
    let endpoints = build_pairing_endpoints(port, tailscale_mode);
    let primary_endpoint = endpoints
        .first()
        .cloned()
        .unwrap_or_else(|| PairingEndpoint {
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

    Ok((pin, pairing_code))
}

fn rotate_pairing_secret_for_port(state: &Arc<SyncServerState>, port: u16, tailscale_mode: &str) -> AppResult<String> {
    let (pin, pairing_code) = generate_pairing_code_for_port(port, tailscale_mode)?;
    {
        let mut inner = state.inner.lock();
        inner.pin = Some(pin);
        inner.pairing_code = Some(pairing_code.clone());
        inner.session_key = generate_session_key();
        inner.failed_attempts.clear();
    }
    Ok(pairing_code)
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
    vault: &Arc<Vault>,
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

    let result = merge_payload(db, vault, &payload);

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

async fn write_json_response(
    stream: &mut tokio::net::TcpStream,
    status: &str,
    body: serde_json::Value,
) {
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
    vault: &Arc<Vault>,
    server_state: &SyncServerState,
    peer_addr: Option<std::net::SocketAddr>,
) {
    let request_str = String::from_utf8_lossy(raw_request);
    let body_start = request_str
        .find("\r\n\r\n")
        .map(|i| i + 4)
        .unwrap_or(raw_request.len());
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

    let payload =
        match export_sync_payload(db, vault).and_then(|payload| serialize_sync_payload(&payload)) {
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

    // Encrypt the session key with a PIN-derived key so only the paired
    // device (which knows the PIN) can decrypt the transfer key.
    let session_key = server_state.inner.lock().session_key;
    let pin_salt = b"shellmate-p2p-pin-salt-v1";
    let response_pin = req.pin.as_deref().unwrap_or(expected_pin);
    let pin_key = {
        let mut key = [0u8; 32];
        let _ = Argon2::default().hash_password_into(response_pin.as_bytes(), pin_salt, &mut key);
        key
    };
    let enc_key = derive_encryption_key(&pin_key);
    let session_key_encrypted = base64::engine::general_purpose::STANDARD
        .encode(encrypt_payload(&enc_key, &session_key).unwrap_or_default());

    // Encrypt the payload JSON with the session key so credentials (now in
    // plaintext) are protected during transport.
    let payload_encrypted = {
        let transfer_key = derive_encryption_key(&session_key);
        let encrypted_bytes = match encrypt_payload(&transfer_key, &payload) {
            Ok(bytes) => bytes,
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
        base64::engine::general_purpose::STANDARD.encode(encrypted_bytes)
    };

    // Build vault key export if vault is unlocked (for mobile auto-setup)
    let vault_key_export = {
        let conn = db.lock();
        let vault_unlocked = vault.is_unlocked();
        if vault_unlocked {
            let get_setting = |key: &str| -> Option<String> {
                conn.query_row("SELECT value FROM settings WHERE key = ?1", [key], |row| {
                    row.get::<_, String>(0)
                })
                .ok()
            };
            let salt = get_setting("vault.salt");
            let verifier_ct = get_setting("vault.verifier.ciphertext");
            let verifier_nonce = get_setting("vault.verifier.nonce");
            if let (Some(salt), Some(ct), Some(nonce)) = (salt, verifier_ct, verifier_nonce) {
                vault.get_vault_key().and_then(|vk| {
                    let transfer_key = derive_encryption_key(&session_key);
                    let export = VaultKeyExport {
                        vault_key: base64::engine::general_purpose::STANDARD.encode(vk),
                        vault_salt: salt,
                        verifier_ct: ct,
                        verifier_nonce: nonce,
                    };
                    let json = serde_json::to_vec(&export).ok()?;
                    let raw = encrypt_payload(&transfer_key, &json).ok()?;
                    Some(base64::engine::general_purpose::STANDARD.encode(raw))
                })
            } else {
                None
            }
        } else {
            None
        }
    };

    let response = PullResponse {
        device_id: req.device_id,
        token: token_to_return,
        payload: payload_encrypted,
        message: "Device paired and synced".into(),
        session_key_encrypted: Some(session_key_encrypted),
        vault_key_export,
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
        "/terminal/spawn" => crate::commands::local_shell::spawn_local_shell(
            app.clone(),
            local_sessions,
            output_buffers,
            req.shell,
        )
        .await
        .map(|session| serde_json::json!({ "session": session })),
        "/terminal/send" => {
            let session_id = req.session_id.unwrap_or_default();
            if local_sessions.contains_key(&session_id) {
                crate::commands::local_shell::send_local_shell(
                    local_sessions,
                    session_id,
                    req.data.unwrap_or_default(),
                )
                .await
                .map(|_| serde_json::json!({ "ok": true }))
            } else {
                use tauri::Manager;
                let app_state = app.state::<AppState>();
                app_state.ssh.send_input(&session_id, req.data.unwrap_or_default().into_bytes())
                    .map(|_| serde_json::json!({ "ok": true }))
            }
        }
        "/terminal/read" => {
            let session_id = req.session_id.unwrap_or_default();
            crate::commands::local_shell::read_local_shell(output_buffers, session_id)
                .await
                .map(|data| serde_json::json!({ "data": data }))
        }
        "/terminal/resize" => {
            let session_id = req.session_id.unwrap_or_default();
            if local_sessions.contains_key(&session_id) {
                crate::commands::local_shell::resize_local_shell(
                    local_sessions,
                    session_id,
                    req.cols.unwrap_or(80),
                    req.rows.unwrap_or(24),
                )
                .await
                .map(|_| serde_json::json!({ "ok": true }))
            } else {
                use tauri::Manager;
                let app_state = app.state::<AppState>();
                app_state.ssh.resize(&session_id, req.cols.unwrap_or(80) as u32, req.rows.unwrap_or(24) as u32)
                    .map(|_| serde_json::json!({ "ok": true }))
            }
        }
        "/terminal/kill" => {
            let session_id = req.session_id.unwrap_or_default();
            if local_sessions.contains_key(&session_id) {
                crate::commands::local_shell::kill_local_shell(
                    local_sessions,
                    output_buffers,
                    session_id,
                )
                .await
                .map(|_| serde_json::json!({ "ok": true }))
            } else {
                use tauri::Manager;
                let app_state = app.state::<AppState>();
                app_state.ssh.close(&session_id)
                    .map(|_| serde_json::json!({ "ok": true }))
            }
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

#[cfg(not(any(target_os = "android", target_os = "ios")))]
async fn handle_terminal_stream_request(
    mut stream: tokio::net::TcpStream,
    raw_request: &[u8],
    db: &Arc<Mutex<Connection>>,
    peer_addr: Option<std::net::SocketAddr>,
) {
    use tokio::io::AsyncWriteExt;

    let Some(req) = parse_terminal_request(&mut stream, raw_request).await else {
        return;
    };

    if let Err(err) = authorize_terminal_request(db, &req, peer_addr) {
        write_json_response(
            &mut stream,
            "401 Unauthorized",
            serde_json::json!({"success": false, "message": err.to_string()}),
        )
        .await;
        return;
    }

    let session_id = req.session_id.unwrap_or_default();
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<crate::commands::local_shell::StreamMessage>();

    // Register the sender
    crate::commands::local_shell::get_terminal_streams().insert(session_id.clone(), tx);

    // Send HTTP/1.1 200 OK with SSE chunked headers
    let response_headers = "HTTP/1.1 200 OK\r\n\
                            Content-Type: text/event-stream\r\n\
                            Cache-Control: no-cache\r\n\
                            Connection: keep-alive\r\n\
                            Transfer-Encoding: chunked\r\n\r\n";
    if stream.write_all(response_headers.as_bytes()).await.is_err() {
        crate::commands::local_shell::get_terminal_streams().remove(&session_id);
        return;
    }
    let _ = stream.flush().await;

    let mut interval = tokio::time::interval(std::time::Duration::from_secs(15));
    loop {
        tokio::select! {
            res = rx.recv() => {
                match res {
                    Some(msg) => {
                        let sse_payload = format!("data: {}\n\n", serde_json::to_string(&msg).unwrap_or_default());
                        let chunk = format!("{:X}\r\n{}\r\n", sse_payload.len(), sse_payload);
                        if stream.write_all(chunk.as_bytes()).await.is_err() {
                            break;
                        }
                        let _ = stream.flush().await;
                    }
                    None => break,
                }
            }
            _ = interval.tick() => {
                let ping_payload = ": ping\n\n";
                let chunk = format!("{:X}\r\n{}\r\n", ping_payload.len(), ping_payload);
                if stream.write_all(chunk.as_bytes()).await.is_err() {
                    break;
                }
                let _ = stream.flush().await;
            }
        }
    }

    let _ = stream.write_all(b"0\r\n\r\n").await;
    let _ = stream.flush().await;

    // Clean up
    crate::commands::local_shell::get_terminal_streams().remove(&session_id);
}

#[cfg(not(any(target_os = "android", target_os = "ios")))]
async fn handle_ssh_connect_request(
    mut stream: tokio::net::TcpStream,
    raw_request: &[u8],
    app: &AppHandle,
    db: &Arc<Mutex<Connection>>,
    peer_addr: Option<std::net::SocketAddr>,
) {
    use tokio::io::AsyncWriteExt;

    let req_body = request_body(raw_request);
    let req: SshConnectRequest = match serde_json::from_slice(req_body) {
        Ok(r) => r,
        Err(_) => {
            write_json_response(
                &mut stream,
                "400 Bad Request",
                serde_json::json!({"success": false, "message": "invalid ssh connect request"}),
            )
            .await;
            return;
        }
    };

    let auth_check = {
        let peer_ip = match peer_addr {
            Some(addr) => addr.ip().to_string(),
            None => {
                write_json_response(
                    &mut stream,
                    "401 Unauthorized",
                    serde_json::json!({"success": false, "message": "missing peer address"}),
                )
                .await;
                return;
            }
        };
        let conn = db.lock();
        conn.query_row(
            "SELECT device_name, bound_ip, revoked_at FROM paired_devices WHERE id = ?1 AND token_hash = ?2",
            rusqlite::params![req.device_id, token_hash(&req.token)],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, Option<String>>(2)?,
                ))
            },
        )
    };

    match auth_check {
        Ok((device_name, bound_ip, revoked_at)) => {
            let peer_ip = peer_addr.unwrap().ip().to_string();
            if revoked_at.is_some() {
                write_json_response(
                    &mut stream,
                    "401 Unauthorized",
                    serde_json::json!({"success": false, "message": "device token revoked"}),
                )
                .await;
                return;
            } else if device_name != req.device_name || bound_ip != peer_ip {
                write_json_response(
                    &mut stream,
                    "401 Unauthorized",
                    serde_json::json!({"success": false, "message": "device identity changed"}),
                )
                .await;
                return;
            }
        }
        Err(_) => {
            write_json_response(
                &mut stream,
                "401 Unauthorized",
                serde_json::json!({"success": false, "message": "unknown device token"}),
            )
            .await;
            return;
        }
    }

    use tauri::Manager;
    let app_state = app.state::<AppState>();

    let connect_input = crate::commands::ssh::ConnectByHostInput {
        host_id: req.host_id,
        session_id: req.session_id,
    };

    match crate::commands::ssh::ssh_connect(app.clone(), app_state, connect_input).await {
        Ok(session_id) => {
            write_json_response(
                &mut stream,
                "200 OK",
                serde_json::json!({ "success": true, "sessionId": session_id }),
            )
            .await;
        }
        Err(err) => {
            write_json_response(
                &mut stream,
                "500 Internal Server Error",
                serde_json::json!({ "success": false, "message": err.to_string() }),
            )
            .await;
        }
    }
}

#[cfg(not(any(target_os = "android", target_os = "ios")))]
async fn handle_ssh_quick_connect_request(
    mut stream: tokio::net::TcpStream,
    raw_request: &[u8],
    app: &AppHandle,
    db: &Arc<Mutex<Connection>>,
    peer_addr: Option<std::net::SocketAddr>,
) {
    use tokio::io::AsyncWriteExt;

    let req_body = request_body(raw_request);
    let req: SshQuickConnectRequest = match serde_json::from_slice(req_body) {
        Ok(r) => r,
        Err(_) => {
            write_json_response(
                &mut stream,
                "400 Bad Request",
                serde_json::json!({"success": false, "message": "invalid ssh quick connect request"}),
            )
            .await;
            return;
        }
    };

    let auth_check = {
        let peer_ip = match peer_addr {
            Some(addr) => addr.ip().to_string(),
            None => {
                write_json_response(
                    &mut stream,
                    "401 Unauthorized",
                    serde_json::json!({"success": false, "message": "missing peer address"}),
                )
                .await;
                return;
            }
        };
        let conn = db.lock();
        conn.query_row(
            "SELECT device_name, bound_ip, revoked_at FROM paired_devices WHERE id = ?1 AND token_hash = ?2",
            rusqlite::params![req.device_id, token_hash(&req.token)],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, Option<String>>(2)?,
                ))
            },
        )
    };

    match auth_check {
        Ok((device_name, bound_ip, revoked_at)) => {
            let peer_ip = peer_addr.unwrap().ip().to_string();
            if revoked_at.is_some() {
                write_json_response(
                    &mut stream,
                    "401 Unauthorized",
                    serde_json::json!({"success": false, "message": "device token revoked"}),
                )
                .await;
                return;
            } else if device_name != req.device_name || bound_ip != peer_ip {
                write_json_response(
                    &mut stream,
                    "401 Unauthorized",
                    serde_json::json!({"success": false, "message": "device identity changed"}),
                )
                .await;
                return;
            }
        }
        Err(_) => {
            write_json_response(
                &mut stream,
                "401 Unauthorized",
                serde_json::json!({"success": false, "message": "unknown device token"}),
            )
            .await;
            return;
        }
    }

    use tauri::Manager;
    let app_state = app.state::<AppState>();

    let connect_input = crate::commands::ssh::QuickConnectInput {
        hostname: req.hostname,
        port: req.port,
        username: req.username,
        label: req.label,
        auth: req.auth,
        shell: req.shell,
        session_id: req.session_id,
    };

    match crate::commands::ssh::ssh_quick_connect(app.clone(), app_state, connect_input).await {
        Ok(session_id) => {
            write_json_response(
                &mut stream,
                "200 OK",
                serde_json::json!({ "success": true, "sessionId": session_id }),
            )
            .await;
        }
        Err(err) => {
            write_json_response(
                &mut stream,
                "500 Internal Server Error",
                serde_json::json!({ "success": false, "message": err.to_string() }),
            )
            .await;
        }
    }
}

#[cfg(not(any(target_os = "android", target_os = "ios")))]
fn capture_primary_screen() -> Option<image::RgbaImage> {
    use xcap::Monitor;
    let monitors = Monitor::all().ok()?;
    let primary = monitors.first()?;
    primary.capture_image().ok()
}

#[cfg(not(any(target_os = "android", target_os = "ios")))]
async fn handle_desktop_screenshot(
    mut stream: tokio::net::TcpStream,
    raw_request: &[u8],
    db: &Arc<Mutex<rusqlite::Connection>>,
    peer_addr: Option<std::net::SocketAddr>,
) {
    let body = request_body(raw_request);
    let auth_req: DesktopAuthRequest = match serde_json::from_slice(body) {
        Ok(r) => r,
        Err(_) => {
            write_json_response(
                &mut stream,
                "400 Bad Request",
                serde_json::json!({"success": false, "message": "invalid desktop request"}),
            )
            .await;
            return;
        }
    };

    let req_payload = TerminalRequest {
        device_id: auth_req.device_id,
        device_name: auth_req.device_name,
        token: auth_req.token,
        session_id: None,
        shell: None,
        data: None,
        cols: None,
        rows: None,
    };
    if let Err(err) = authorize_terminal_request(db, &req_payload, peer_addr) {
        write_json_response(
            &mut stream,
            "401 Unauthorized",
            serde_json::json!({"success": false, "message": err.to_string()}),
        )
        .await;
        return;
    }

    let image = match capture_primary_screen() {
        Some(img) => img,
        None => {
            write_json_response(
                &mut stream,
                "500 Internal Server Error",
                serde_json::json!({"success": false, "message": "failed to capture screen"}),
            )
            .await;
            return;
        }
    };

    let mut jpeg_data = Vec::new();
    let mut cursor = std::io::Cursor::new(&mut jpeg_data);
    if image.write_to(&mut cursor, image::ImageFormat::Jpeg).is_err() {
        write_json_response(
            &mut stream,
            "500 Internal Server Error",
            serde_json::json!({"success": false, "message": "failed to encode image"}),
        )
        .await;
        return;
    }

    let response_headers = format!(
        "HTTP/1.1 200 OK\r\n\
         Content-Type: image/jpeg\r\n\
         Content-Length: {}\r\n\
         Connection: close\r\n\r\n",
        jpeg_data.len()
    );
    if stream.write_all(response_headers.as_bytes()).await.is_ok() {
        let _ = stream.write_all(&jpeg_data).await;
        let _ = stream.flush().await;
    }
}

#[cfg(not(any(target_os = "android", target_os = "ios")))]
async fn handle_desktop_input(
    mut stream: tokio::net::TcpStream,
    raw_request: &[u8],
    db: &Arc<Mutex<rusqlite::Connection>>,
    peer_addr: Option<std::net::SocketAddr>,
) {
    use enigo::{Enigo, Mouse, Keyboard, Settings, Coordinate};

    let body = request_body(raw_request);
    let input_req: DesktopInputRequest = match serde_json::from_slice(body) {
        Ok(r) => r,
        Err(_) => {
            write_json_response(
                &mut stream,
                "400 Bad Request",
                serde_json::json!({"success": false, "message": "invalid input request"}),
            )
            .await;
            return;
        }
    };

    let req_payload = TerminalRequest {
        device_id: input_req.device_id,
        device_name: input_req.device_name,
        token: input_req.token,
        session_id: None,
        shell: None,
        data: None,
        cols: None,
        rows: None,
    };
    if let Err(err) = authorize_terminal_request(db, &req_payload, peer_addr) {
        write_json_response(
            &mut stream,
            "401 Unauthorized",
            serde_json::json!({"success": false, "message": err.to_string()}),
        )
        .await;
        return;
    }

    let mut enigo = match Enigo::new(&Settings::default()) {
        Ok(e) => e,
        Err(e) => {
            write_json_response(
                &mut stream,
                "500 Internal Server Error",
                serde_json::json!({"success": false, "message": e.to_string()}),
            )
            .await;
            return;
        }
    };

    match input_req.event.as_str() {
        "move" => {
            let x = input_req.x.unwrap_or(0);
            let y = input_req.y.unwrap_or(0);
            let _ = enigo.move_mouse(x, y, Coordinate::Abs);
        }
        "click" => {
            let button = match input_req.button.as_deref() {
                Some("right") => enigo::Button::Right,
                Some("middle") => enigo::Button::Middle,
                _ => enigo::Button::Left,
            };
            let _ = enigo.button(button, enigo::Direction::Click);
        }
        "keydown" | "keyup" => {
            if let Some(ref key_str) = input_req.key {
                let key = parse_enigo_key(key_str);
                let dir = if input_req.event == "keydown" {
                    enigo::Direction::Press
                } else {
                    enigo::Direction::Release
                };
                let _ = enigo.key(key, dir);
            }
        }
        _ => {}
    }

    write_json_response(&mut stream, "200 OK", serde_json::json!({"success": true})).await;
}

#[cfg(not(any(target_os = "android", target_os = "ios")))]
fn parse_enigo_key(key_str: &str) -> enigo::Key {
    match key_str {
        "Backspace" => enigo::Key::Backspace,
        "Tab" => enigo::Key::Tab,
        "Enter" => enigo::Key::Return,
        "Escape" => enigo::Key::Escape,
        "Space" => enigo::Key::Space,
        "ArrowLeft" => enigo::Key::LeftArrow,
        "ArrowRight" => enigo::Key::RightArrow,
        "ArrowUp" => enigo::Key::UpArrow,
        "ArrowDown" => enigo::Key::DownArrow,
        other if other.len() == 1 => enigo::Key::Unicode(other.chars().next().unwrap()),
        _ => enigo::Key::Space,
    }
}

#[cfg(not(any(target_os = "android", target_os = "ios")))]
async fn handle_desktop_stream(
    mut stream: tokio::net::TcpStream,
    raw_request: &[u8],
    db: &Arc<Mutex<rusqlite::Connection>>,
    peer_addr: Option<std::net::SocketAddr>,
) {
    use xcap::Monitor;
    use tokio::io::AsyncWriteExt;
    use base64::Engine;

    let body = request_body(raw_request);
    let auth_req: DesktopAuthRequest = match serde_json::from_slice(body) {
        Ok(r) => r,
        Err(_) => {
            write_json_response(&mut stream, "400 Bad Request", serde_json::json!({"success": false})).await;
            return;
        }
    };

    let req_payload = TerminalRequest {
        device_id: auth_req.device_id,
        device_name: auth_req.device_name,
        token: auth_req.token,
        session_id: None,
        shell: None,
        data: None,
        cols: None,
        rows: None,
    };
    if authorize_terminal_request(db, &req_payload, peer_addr).is_err() {
        write_json_response(&mut stream, "401 Unauthorized", serde_json::json!({"success": false})).await;
        return;
    }

    let response_headers = "HTTP/1.1 200 OK\r\n\
                            Content-Type: text/event-stream\r\n\
                            Cache-Control: no-cache\r\n\
                            Connection: keep-alive\r\n\
                            Transfer-Encoding: chunked\r\n\r\n";
    if stream.write_all(response_headers.as_bytes()).await.is_err() {
        return;
    }
    let _ = stream.flush().await;

    let mut interval = tokio::time::interval(std::time::Duration::from_millis(200));
    loop {
        tokio::select! {
            _ = interval.tick() => {
                let image = match capture_primary_screen() {
                    Some(img) => img,
                    None => break,
                };

                let mut jpeg_data = Vec::new();
                let mut cursor = std::io::Cursor::new(&mut jpeg_data);
                if image.write_to(&mut cursor, image::ImageFormat::Jpeg).is_err() {
                    continue;
                }

                let base64_data = base64::engine::general_purpose::STANDARD.encode(&jpeg_data);
                let sse_payload = format!("data: {}\n\n", serde_json::json!({
                    "type": "desktop",
                    "image": base64_data,
                }).to_string());
                let chunk = format!("{:X}\r\n{}\r\n", sse_payload.len(), sse_payload);
                if stream.write_all(chunk.as_bytes()).await.is_err() {
                    break;
                }
                let _ = stream.flush().await;
            }
        }
    }

    let _ = stream.write_all(b"0\r\n\r\n").await;
    let _ = stream.flush().await;
}

#[cfg(not(any(target_os = "android", target_os = "ios")))]
async fn handle_file_list(
    mut stream: tokio::net::TcpStream,
    raw_request: &[u8],
    db: &Arc<Mutex<rusqlite::Connection>>,
    peer_addr: Option<std::net::SocketAddr>,
) {
    use tokio::io::AsyncWriteExt;

    let body = request_body(raw_request);
    let list_req: FileListRequest = match serde_json::from_slice(body) {
        Ok(r) => r,
        Err(_) => {
            write_json_response(
                &mut stream,
                "400 Bad Request",
                serde_json::json!({"success": false, "message": "invalid file list request"}),
            )
            .await;
            return;
        }
    };

    let req_payload = TerminalRequest {
        device_id: list_req.device_id,
        device_name: list_req.device_name,
        token: list_req.token,
        session_id: None,
        shell: None,
        data: None,
        cols: None,
        rows: None,
    };
    if let Err(err) = authorize_terminal_request(db, &req_payload, peer_addr) {
        write_json_response(
            &mut stream,
            "401 Unauthorized",
            serde_json::json!({"success": false, "message": err.to_string()}),
        )
        .await;
        return;
    }

    let target_path = list_req.path.clone().unwrap_or_else(|| {
        dirs::home_dir()
            .map(|p| p.to_string_lossy().into_owned())
            .unwrap_or_else(|| "/".into())
    });

    let mut items = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&target_path) {
        for entry in entries.flatten() {
            if let Ok(meta) = entry.metadata() {
                items.push(serde_json::json!({
                    "name": entry.file_name().to_string_lossy(),
                    "size": meta.len(),
                    "isDir": meta.is_dir(),
                    "modified": meta.modified().ok()
                        .and_then(|t| t.duration_since(std::time::SystemTime::UNIX_EPOCH).ok())
                        .map(|d| d.as_secs()),
                }));
            }
        }
    }

    write_json_response(
        &mut stream,
        "200 OK",
        serde_json::json!({
            "success": true,
            "currentPath": target_path,
            "items": items
        }),
    )
    .await;
}

#[cfg(not(any(target_os = "android", target_os = "ios")))]
async fn handle_file_download(
    mut stream: tokio::net::TcpStream,
    raw_request: &[u8],
    db: &Arc<Mutex<rusqlite::Connection>>,
    peer_addr: Option<std::net::SocketAddr>,
) {
    use tokio::io::AsyncWriteExt;

    let body = request_body(raw_request);
    let dl_req: FileDownloadRequest = match serde_json::from_slice(body) {
        Ok(r) => r,
        Err(_) => {
            write_json_response(
                &mut stream,
                "400 Bad Request",
                serde_json::json!({"success": false, "message": "invalid download request"}),
            )
            .await;
            return;
        }
    };

    let req_payload = TerminalRequest {
        device_id: dl_req.device_id,
        device_name: dl_req.device_name,
        token: dl_req.token,
        session_id: None,
        shell: None,
        data: None,
        cols: None,
        rows: None,
    };
    if let Err(err) = authorize_terminal_request(db, &req_payload, peer_addr) {
        write_json_response(
            &mut stream,
            "401 Unauthorized",
            serde_json::json!({"success": false, "message": err.to_string()}),
        )
        .await;
        return;
    }

    let file_path = std::path::Path::new(&dl_req.path);
    if !file_path.is_file() {
        write_json_response(
            &mut stream,
            "404 Not Found",
            serde_json::json!({"success": false, "message": "file not found"}),
        )
        .await;
        return;
    }

    let file_size = match std::fs::metadata(file_path) {
        Ok(m) => m.len(),
        Err(e) => {
            write_json_response(
                &mut stream,
                "500 Internal Error",
                serde_json::json!({"success": false, "message": e.to_string()}),
            )
            .await;
            return;
        }
    };

    let mut file = match tokio::fs::File::open(file_path).await {
        Ok(f) => f,
        Err(e) => {
            write_json_response(
                &mut stream,
                "500 Internal Error",
                serde_json::json!({"success": false, "message": e.to_string()}),
            )
            .await;
            return;
        }
    };

    let filename = file_path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| "file".to_string());

    let headers = format!(
        "HTTP/1.1 200 OK\r\n\
         Content-Type: application/octet-stream\r\n\
         Content-Length: {}\r\n\
         Content-Disposition: attachment; filename=\"{}\"\r\n\
         Connection: close\r\n\r\n",
        file_size, filename
    );

    if stream.write_all(headers.as_bytes()).await.is_err() {
        return;
    }

    let mut buffer = [0u8; 8192];
    loop {
        match tokio::io::AsyncReadExt::read(&mut file, &mut buffer).await {
            Ok(0) => break,
            Ok(n) => {
                if stream.write_all(&buffer[..n]).await.is_err() {
                    break;
                }
            }
            Err(_) => break,
        }
    }
    let _ = stream.flush().await;
}

#[cfg(not(any(target_os = "android", target_os = "ios")))]
async fn handle_file_upload(
    mut stream: tokio::net::TcpStream,
    initial_buf: &[u8],
    db: &Arc<Mutex<rusqlite::Connection>>,
    peer_addr: Option<std::net::SocketAddr>,
) {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    let request_str = String::from_utf8_lossy(initial_buf);
    let path = get_http_header(&request_str, "X-File-Path");
    let device_id = get_http_header(&request_str, "X-Device-Id");
    let device_name = get_http_header(&request_str, "X-Device-Name");
    let token = get_http_header(&request_str, "X-Token");
    let content_length_str = get_http_header(&request_str, "Content-Length");

    let (path, device_id, device_name, token, content_length) = match (path, device_id, device_name, token, content_length_str) {
        (Some(p), Some(did), Some(dname), Some(tok), Some(cl)) => {
            let cl_val = cl.parse::<usize>().unwrap_or(0);
            (p, did, dname, tok, cl_val)
        }
        _ => {
            write_json_response(
                &mut stream,
                "400 Bad Request",
                serde_json::json!({"success": false, "message": "missing required headers"}),
            )
            .await;
            return;
        }
    };

    let req_payload = TerminalRequest {
        device_id,
        device_name,
        token,
        session_id: None,
        shell: None,
        data: None,
        cols: None,
        rows: None,
    };
    if let Err(err) = authorize_terminal_request(db, &req_payload, peer_addr) {
        write_json_response(
            &mut stream,
            "401 Unauthorized",
            serde_json::json!({"success": false, "message": err.to_string()}),
        )
        .await;
        return;
    }

    let file_path = std::path::Path::new(&path);
    if let Some(parent) = file_path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }

    let mut file = match tokio::fs::File::create(file_path).await {
        Ok(f) => f,
        Err(e) => {
            write_json_response(
                &mut stream,
                "500 Internal Error",
                serde_json::json!({"success": false, "message": e.to_string()}),
            )
            .await;
            return;
        }
    };

    let body_start = request_str.find("\r\n\r\n").unwrap_or(0) + 4;
    let initial_data = &initial_buf[body_start..];
    let mut written = initial_data.len();

    if file.write_all(initial_data).await.is_err() {
        write_json_response(
            &mut stream,
            "500 Internal Error",
            serde_json::json!({"success": false, "message": "failed to write initial data"}),
        )
        .await;
        return;
    }

    let mut buffer = [0u8; 8192];
    while written < content_length {
        match stream.read(&mut buffer).await {
            Ok(0) => break,
            Ok(n) => {
                if file.write_all(&buffer[..n]).await.is_err() {
                    break;
                }
                written += n;
            }
            Err(_) => break,
        }
    }

    let _ = file.flush().await;

    write_json_response(&mut stream, "200 OK", serde_json::json!({"success": true})).await;
}

#[cfg(not(any(target_os = "android", target_os = "ios")))]
async fn handle_file_delete(
    mut stream: tokio::net::TcpStream,
    raw_request: &[u8],
    db: &Arc<Mutex<rusqlite::Connection>>,
    peer_addr: Option<std::net::SocketAddr>,
) {
    use tokio::io::AsyncWriteExt;

    let body = request_body(raw_request);
    let del_req: FileDeleteRequest = match serde_json::from_slice(body) {
        Ok(r) => r,
        Err(_) => {
            write_json_response(
                &mut stream,
                "400 Bad Request",
                serde_json::json!({"success": false, "message": "invalid delete request"}),
            )
            .await;
            return;
        }
    };

    let req_payload = TerminalRequest {
        device_id: del_req.device_id,
        device_name: del_req.device_name,
        token: del_req.token,
        session_id: None,
        shell: None,
        data: None,
        cols: None,
        rows: None,
    };
    if let Err(err) = authorize_terminal_request(db, &req_payload, peer_addr) {
        write_json_response(
            &mut stream,
            "401 Unauthorized",
            serde_json::json!({"success": false, "message": err.to_string()}),
        )
        .await;
        return;
    }

    let file_path = std::path::Path::new(&del_req.path);
    if !file_path.exists() {
        write_json_response(
            &mut stream,
            "404 Not Found",
            serde_json::json!({"success": false, "message": "path not found"}),
        )
        .await;
        return;
    }

    let res = if file_path.is_dir() {
        std::fs::remove_dir_all(file_path)
    } else {
        std::fs::remove_file(file_path)
    };

    match res {
        Ok(_) => {
            write_json_response(&mut stream, "200 OK", serde_json::json!({"success": true})).await;
        }
        Err(e) => {
            write_json_response(
                &mut stream,
                "500 Internal Error",
                serde_json::json!({"success": false, "message": e.to_string()}),
            )
            .await;
        }
    }
}

#[cfg(not(any(target_os = "android", target_os = "ios")))]
async fn handle_local_proxy(
    mut stream: tokio::net::TcpStream,
    initial_buf: &[u8],
    method: &str,
    path: &str,
    db: &Arc<Mutex<rusqlite::Connection>>,
    peer_addr: Option<std::net::SocketAddr>,
) {
    use tokio::io::AsyncWriteExt;

    let request_str = String::from_utf8_lossy(initial_buf);
    let device_id = get_http_header(&request_str, "X-Device-Id");
    let device_name = get_http_header(&request_str, "X-Device-Name");
    let token = get_http_header(&request_str, "X-Token");

    let authorized = match (device_id, device_name, token) {
        (Some(did), Some(dname), Some(tok)) => {
            let req_payload = TerminalRequest {
                device_id: did,
                device_name: dname,
                token: tok,
                session_id: None,
                shell: None,
                data: None,
                cols: None,
                rows: None,
            };
            authorize_terminal_request(db, &req_payload, peer_addr).is_ok()
        }
        _ => false,
    };

    if !authorized {
        let response = "HTTP/1.1 401 Unauthorized\r\nContent-Length: 0\r\n\r\n";
        let _ = stream.write_all(response.as_bytes()).await;
        return;
    }

    let path_parts: Vec<&str> = path.split('/').collect();
    if path_parts.len() >= 3 {
        let port_str = path_parts[2];
        if let Ok(target_port) = port_str.parse::<u16>() {
            let subpath = path_parts[3..].join("/");
            let target_url = format!("http://localhost:{}/{}", target_port, subpath);

            let client = get_reqwest_client();
            let method_upper = method.to_uppercase();
            let req_body = request_body(initial_buf);

            let req_method = match method_upper.as_str() {
                "GET" => reqwest::Method::GET,
                "POST" => reqwest::Method::POST,
                "PUT" => reqwest::Method::PUT,
                "DELETE" => reqwest::Method::DELETE,
                "PATCH" => reqwest::Method::PATCH,
                "OPTIONS" => reqwest::Method::OPTIONS,
                "HEAD" => reqwest::Method::HEAD,
                _ => reqwest::Method::GET,
            };

            let mut req_builder = client.request(req_method, &target_url);
            if !req_body.is_empty() {
                req_builder = req_builder.body(req_body.to_vec());
            }

            match req_builder.send().await {
                Ok(resp) => {
                    let status = resp.status();
                    let headers = resp.headers().clone();
                    let resp_bytes = resp.bytes().await.unwrap_or_default();

                    let mut resp_str = format!("HTTP/1.1 {}\r\n", status);
                    for (k, v) in headers.iter() {
                        let key_lower = k.as_str().to_lowercase();
                        if key_lower != "transfer-encoding" && key_lower != "content-length" {
                            if let Ok(v_str) = v.to_str() {
                                resp_str.push_str(&format!("{}: {}\r\n", k.as_str(), v_str));
                            }
                        }
                    }
                    resp_str.push_str(&format!("Content-Length: {}\r\n\r\n", resp_bytes.len()));

                    if stream.write_all(resp_str.as_bytes()).await.is_ok() {
                        let _ = stream.write_all(&resp_bytes).await;
                        let _ = stream.flush().await;
                    }
                }
                Err(e) => {
                    let err_body = serde_json::json!({"success": false, "error": e.to_string()}).to_string();
                    let response = format!(
                        "HTTP/1.1 502 Bad Gateway\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
                        err_body.len(), err_body
                    );
                    let _ = stream.write_all(response.as_bytes()).await;
                }
            }
            return;
        }
    }

    let response = "HTTP/1.1 400 Bad Request\r\nContent-Length: 0\r\n\r\n";
    let _ = stream.write_all(response.as_bytes()).await;
}

fn merge_payload(
    db: &Arc<Mutex<Connection>>,
    vault: &Arc<Vault>,
    payload: &SyncPayload,
) -> AppResult<String> {
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

    let vault_unlocked = vault.is_unlocked();

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
            // If the payload is NOT encrypted (credentials were decrypted with
            // the desktop vault and sent as plaintext), re-encrypt with the
            // local vault before storing. Otherwise, store the raw ciphertext
            // from the desktop (backward compat — will fail on decrypt with
            // local vault unless both devices share the same master password).
            if !payload.encrypted {
                if let (true, Some(ref plaintext)) = (vault_unlocked, &cred.plaintext) {
                    let blob = vault.encrypt(plaintext.as_bytes()).map_err(|e| {
                        AppError::Internal(format!("credential re-encrypt failed: {e}"))
                    })?;
                    tx.execute(
                        "INSERT INTO credentials (id, type, encrypted_data, nonce, created_at, updated_at)
                         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                        rusqlite::params![
                            cred.id,
                            cred.cred_type,
                            blob.ciphertext,
                            blob.nonce.to_vec(),
                            cred.created_at,
                            cred.updated_at
                        ],
                    )?;
                } else {
                    // Vault locked or missing plaintext — store empty blob;
                    // credential will need to be re-entered by user.
                    tx.execute(
                        "INSERT INTO credentials (id, type, encrypted_data, nonce, created_at, updated_at)
                         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                        rusqlite::params![
                            cred.id,
                            cred.cred_type,
                            Vec::<u8>::new(),
                            vec![0u8; 12],
                            cred.created_at,
                            cred.updated_at
                        ],
                    )?;
                }
            } else {
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
            }
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

                            #[cfg(not(any(target_os = "android", target_os = "ios")))]
                            if path.starts_with("/proxy/") {
                                handle_local_proxy(stream, &buf, &method, &path, &db, peer_addr).await;
                                return;
                            }

                            match (method.as_str(), path.as_str()) {
                                ("GET", "/pair") => {
                                    // Return server info + session key encrypted with PIN.
                                    // PIN is NOT returned — client must know it from server display.
                                    let session_key_encrypted = {
                                        let (pin, session_key) = {
                                            let inner = server_state_ref.inner.lock();
                                            (inner.pin.clone(), inner.session_key)
                                        };
                                        let Some(pin) = pin else {
                                            let response = "HTTP/1.1 409 Conflict\r\nContent-Length: 0\r\n\r\n";
                                            let _ = stream.write_all(response.as_bytes()).await;
                                            return;
                                        };
                                        let pin_salt = b"shellmate-p2p-pin-salt-v1";
                                        let mut pin_key = [0u8; 32];
                                        let _ = Argon2::default().hash_password_into(pin.as_bytes(), pin_salt, &mut pin_key);
                                        let enc_key = derive_encryption_key(&pin_key);
                                        encrypt_payload(&enc_key, &session_key).ok()
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
                                    let expected_pin = {
                                        let inner = server_state_ref.inner.lock();
                                        inner.pin.clone()
                                    };
                                    let Some(expected_pin) = expected_pin else {
                                        let response = "HTTP/1.1 409 Conflict\r\nContent-Length: 0\r\n\r\n";
                                        let _ = stream.write_all(response.as_bytes()).await;
                                        return;
                                    };
                                    handle_pair_pull(
                                        &mut stream,
                                        &buf,
                                        &expected_pin,
                                        &db,
                                        &vault,
                                        &server_state_ref,
                                        peer_addr,
                                    ).await;
                                }
                                ("POST", "/sync") => {
                                    let expected_pin = {
                                        let inner = server_state_ref.inner.lock();
                                        inner.pin.clone()
                                    };
                                    let Some(expected_pin) = expected_pin else {
                                        let response = "HTTP/1.1 409 Conflict\r\nContent-Length: 0\r\n\r\n";
                                        let _ = stream.write_all(response.as_bytes()).await;
                                        return;
                                    };
                                    handle_sync_receive(
                                        &mut stream,
                                        &buf,
                                        &expected_pin,
                                        &db,
                                        &vault,
                                        &app,
                                        peer_addr,
                                        &server_state_ref,
                                    ).await;
                                }
                                #[cfg(not(any(target_os = "android", target_os = "ios")))]
                                ("POST", "/terminal/stream") => {
                                    handle_terminal_stream_request(
                                        stream,
                                        &buf,
                                        &db,
                                        peer_addr,
                                    )
                                    .await;
                                }
                                #[cfg(not(any(target_os = "android", target_os = "ios")))]
                                ("POST", "/ssh/connect") => {
                                    handle_ssh_connect_request(
                                        stream,
                                        &buf,
                                        &app,
                                        &db,
                                        peer_addr,
                                    )
                                    .await;
                                }
                                #[cfg(not(any(target_os = "android", target_os = "ios")))]
                                ("POST", "/ssh/quick_connect") => {
                                    handle_ssh_quick_connect_request(
                                        stream,
                                        &buf,
                                        &app,
                                        &db,
                                        peer_addr,
                                    )
                                    .await;
                                }
                                #[cfg(not(any(target_os = "android", target_os = "ios")))]
                                ("POST", "/desktop/screenshot") => {
                                    handle_desktop_screenshot(
                                        stream,
                                        &buf,
                                        &db,
                                        peer_addr,
                                    )
                                    .await;
                                }
                                #[cfg(not(any(target_os = "android", target_os = "ios")))]
                                ("POST", "/desktop/input") => {
                                    handle_desktop_input(
                                        stream,
                                        &buf,
                                        &db,
                                        peer_addr,
                                    )
                                    .await;
                                }
                                #[cfg(not(any(target_os = "android", target_os = "ios")))]
                                ("POST", "/desktop/stream") => {
                                    handle_desktop_stream(
                                        stream,
                                        &buf,
                                        &db,
                                        peer_addr,
                                    )
                                    .await;
                                }
                                #[cfg(not(any(target_os = "android", target_os = "ios")))]
                                ("POST", "/files/list") => {
                                    handle_file_list(
                                        stream,
                                        &buf,
                                        &db,
                                        peer_addr,
                                    )
                                    .await;
                                }
                                #[cfg(not(any(target_os = "android", target_os = "ios")))]
                                ("POST", "/files/download") => {
                                    handle_file_download(
                                        stream,
                                        &buf,
                                        &db,
                                        peer_addr,
                                    )
                                    .await;
                                }
                                #[cfg(not(any(target_os = "android", target_os = "ios")))]
                                ("POST", "/files/upload") => {
                                    handle_file_upload(
                                        stream,
                                        &buf,
                                        &db,
                                        peer_addr,
                                    )
                                    .await;
                                }
                                #[cfg(not(any(target_os = "android", target_os = "ios")))]
                                ("POST", "/files/delete") => {
                                    handle_file_delete(
                                        stream,
                                        &buf,
                                        &db,
                                        peer_addr,
                                    )
                                    .await;
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
    inner.port = None;
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
    let running_port = {
        let inner = state.inner.lock();
        if inner.is_running {
            Some(inner.port.ok_or_else(|| {
                AppError::Internal("sync server is running without a port".into())
            })?)
        } else {
            None
        }
    };

    let tailscale_mode = {
        let conn = db.lock();
        conn.query_row(
            "SELECT value FROM settings WHERE key = 'p2p.tailscale_mode'",
            [],
            |row| row.get::<_, String>(0),
        )
        .unwrap_or_else(|_| "off".to_string())
    };

    if let Some(port) = running_port {
        let pairing_code = rotate_pairing_secret_for_port(&state, port, &tailscale_mode)?;
        let _ = app.emit("p2p:pairing-ready", &pairing_code);
        return Ok(pairing_code);
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
    let (pin, pairing_code) = generate_pairing_code_for_port(port, &tailscale_mode)?;

    {
        let mut inner = state.inner.lock();
        inner.is_running = true;
        inner.pin = Some(pin.clone());
        inner.pairing_code = Some(pairing_code.clone());
        inner.port = Some(port);
        inner.session_key = generate_session_key();
        inner.failed_attempts.clear();
        inner.shutdown_tx = Some(shutdown_tx);
    }

    let server_state = Arc::clone(&state);

    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    if tailscale_mode == "serve" || tailscale_mode == "funnel" {
        if let Err(e) = tailscale_start_serve(port, tailscale_mode == "funnel") {
            log::error!("Failed to start tailscale serve/funnel: {}", e);
        }
    }

    tokio::spawn(run_server(
        app,
        listener,
        port,
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
    let port = inner.port;
    if let Some(tx) = inner.shutdown_tx.take() {
        let _ = tx.send(());
    }
    inner.is_running = false;
    inner.pin = None;
    inner.pairing_code = None;

    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    if let Some(p) = port {
        tailscale_stop_serve(p);
    }
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
    encode_sync_payload(&export_sync_payload(&app_state.db, &app_state.vault)?)
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
    app: AppHandle,
    state: State<'_, Arc<SyncServerState>>,
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

    let rotated_pairing_code = {
        let port = {
            let inner = state.inner.lock();
            if inner.is_running {
                inner.port
            } else {
                None
            }
        };
        let tailscale_mode = {
            let conn = app_state.db.lock();
            conn.query_row(
                "SELECT value FROM settings WHERE key = 'p2p.tailscale_mode'",
                [],
                |row| row.get::<_, String>(0),
            )
            .unwrap_or_else(|_| "off".to_string())
        };
        match port {
            Some(port) => Some(rotate_pairing_secret_for_port(&state, port, &tailscale_mode)?),
            None => None,
        }
    };

    if let Some(pairing_code) = rotated_pairing_code {
        let _ = app.emit("p2p:pairing-ready", &pairing_code);
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
        wants_unlock: false,
    };

    let (pull_response, endpoint) =
        request_desktop_pull(&client, &endpoints, &pull, "desktop pairing").await?;

    // Decrypt the payload. If the desktop sent session_key_encrypted (v2+),
    // use session-key decryption. Otherwise fall back to raw base64 payload
    // (backward compat with older desktop versions).
    let (payload, saved_session_key): (SyncPayload, Option<String>) =
        if let Some(ref sk_enc) = pull_response.session_key_encrypted {
            let pin_salt = b"shellmate-p2p-pin-salt-v1";
            let pin_key = {
                let mut key = [0u8; 32];
                Argon2::default()
                    .hash_password_into(code.pin.as_bytes(), pin_salt, &mut key)
                    .map_err(|_| AppError::InvalidInput("PIN key derivation failed".into()))?;
                key
            };
            let enc_key = derive_encryption_key(&pin_key);
            let sk_bytes = base64::engine::general_purpose::STANDARD
                .decode(sk_enc.as_bytes())
                .map_err(|_| AppError::InvalidInput("invalid session key encoding".into()))?;
            let session_key = {
                let data = decrypt_payload(&enc_key, &sk_bytes)?;
                if data.len() != 32 {
                    return Err(AppError::InvalidInput("session key length mismatch".into()));
                }
                let mut k = [0u8; 32];
                k.copy_from_slice(&data);
                k
            };

            // Save session key for later token-based re-syncs that don't have the PIN
            let saved = base64::engine::general_purpose::STANDARD.encode(&session_key);

            let transfer_key = derive_encryption_key(&session_key);
            let encrypted_bytes = base64::engine::general_purpose::STANDARD
                .decode(pull_response.payload.as_bytes())
                .map_err(|_| AppError::InvalidInput("invalid encrypted payload".into()))?;
            let payload_json = decrypt_payload(&transfer_key, &encrypted_bytes)?;
            let payload = serde_json::from_slice(&payload_json)
                .map_err(|e| AppError::InvalidInput(format!("invalid sync payload: {e}")))?;
            (payload, Some(saved))
        } else {
            // Legacy: raw base64-encoded JSON payload (no transport encryption)
            let payload_bytes = base64::engine::general_purpose::STANDARD
                .decode(pull_response.payload.as_bytes())
                .map_err(|_| AppError::InvalidInput("invalid sync payload".into()))?;
            let payload = serde_json::from_slice(&payload_bytes)
                .map_err(|e| AppError::InvalidInput(format!("invalid sync payload: {e}")))?;
            (payload, None)
        };

    // Import vault key from desktop if provided (satellite device mode)
    // Must happen before merge so credentials can be re-encrypted with the
    // imported desktop vault key instead of being stored as unusable blanks.
    import_vault_key_export(&app_state, &pull_response, &code)?;

    let message = merge_payload(&app_state.db, &app_state.vault, &payload)?;

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
        if let Some(ref sk) = saved_session_key {
            conn.execute(
                "INSERT INTO settings (key, value) VALUES ('p2p.session_key', ?1)
                 ON CONFLICT(key) DO UPDATE SET value = excluded.value",
                [sk],
            )?;
        }
    }
    Ok(format!("{}; {}", pull_response.message, message))
}

fn import_vault_key_export(
    app_state: &AppState,
    pull_response: &PullResponse,
    code: &PairingCode,
) -> AppResult<()> {
    let Some(ref vk_enc) = pull_response.vault_key_export else {
        return Ok(());
    };

    // Decrypt vault key export using session key (derived from PIN)
    let session_key = {
        let pin_salt = b"shellmate-p2p-pin-salt-v1";
        let mut pin_key = [0u8; 32];
        Argon2::default()
            .hash_password_into(code.pin.as_bytes(), pin_salt, &mut pin_key)
            .map_err(|_| AppError::InvalidInput("PIN key derivation failed".into()))?;
        let enc_key = derive_encryption_key(&pin_key);
        let sk_enc = base64::engine::general_purpose::STANDARD
            .decode(
                pull_response
                    .session_key_encrypted
                    .as_ref()
                    .ok_or_else(|| AppError::InvalidInput("missing session key".into()))?
                    .as_bytes(),
            )
            .map_err(|_| AppError::InvalidInput("invalid session key encoding".into()))?;
        let data = decrypt_payload(&enc_key, &sk_enc)?;
        if data.len() != 32 {
            return Err(AppError::InvalidInput("session key length mismatch".into()));
        }
        let mut k = [0u8; 32];
        k.copy_from_slice(&data);
        k
    };

    let transfer_key = derive_encryption_key(&session_key);
    let encrypted_bytes = base64::engine::general_purpose::STANDARD
        .decode(vk_enc.as_bytes())
        .map_err(|_| AppError::InvalidInput("invalid vault key export".into()))?;
    let json = decrypt_payload(&transfer_key, &encrypted_bytes)?;
    let export: VaultKeyExport = serde_json::from_slice(&json)
        .map_err(|_| AppError::InvalidInput("invalid vault key export format".into()))?;

    let vault_key_bytes = base64::engine::general_purpose::STANDARD
        .decode(export.vault_key.as_bytes())
        .map_err(|_| AppError::InvalidInput("invalid vault key encoding".into()))?;
    if vault_key_bytes.len() != 32 {
        return Err(AppError::InvalidInput("vault key length mismatch".into()));
    }

    // Save vault metadata and key to mobile DB
    let conn = app_state.db.lock();
    conn.execute(
        "INSERT OR REPLACE INTO settings (key, value) VALUES ('vault.initialized', '1')",
        [],
    )?;
    conn.execute(
        "INSERT OR REPLACE INTO settings (key, value) VALUES ('vault.salt', ?1)",
        [&export.vault_salt],
    )?;
    conn.execute(
        "INSERT OR REPLACE INTO settings (key, value) VALUES ('vault.verifier.ciphertext', ?1)",
        [&export.verifier_ct],
    )?;
    conn.execute(
        "INSERT OR REPLACE INTO settings (key, value) VALUES ('vault.verifier.nonce', ?1)",
        [&export.verifier_nonce],
    )?;
    // Store encrypted vault key for auto-unlock (encrypted with device-specific key)
    let device_id: String = conn.query_row(
        "SELECT value FROM settings WHERE key = 'p2p.device_id'",
        [],
        |row| row.get(0),
    )?;
    use sha2::{Digest, Sha256};
    let device_enc_key: [u8; 32] =
        Sha256::digest(format!("shellmate-auto-unlock-{device_id}").as_bytes()).into();
    let enc_blob = crate::crypto::encrypt(&device_enc_key, &vault_key_bytes)?;
    conn.execute(
        "INSERT OR REPLACE INTO settings (key, value) VALUES ('p2p.vault_key_encrypted', ?1)",
        [&base64::engine::general_purpose::STANDARD.encode(&enc_blob.ciphertext)],
    )?;
    conn.execute(
        "INSERT OR REPLACE INTO settings (key, value) VALUES ('p2p.vault_key_nonce', ?1)",
        [&hex::encode(enc_blob.nonce)],
    )?;
    drop(conn);

    // Unlock vault with the imported vault key
    let mut vk = [0u8; 32];
    vk.copy_from_slice(&vault_key_bytes);
    app_state.vault.unlock_with_vault_key(&vk)?;

    log::info!("Imported vault key from desktop — mobile vault unlocked");
    Ok(())
}

#[tauri::command]
pub async fn p2p_auto_unlock(app_state: State<'_, AppState>) -> AppResult<String> {
    // Try to unlock vault from stored encrypted key (from previous pairing)
    let (vault_key_ct, vault_key_nonce, device_id) = {
        let conn = app_state.db.lock();
        let get = |key: &str| -> Option<String> {
            conn.query_row("SELECT value FROM settings WHERE key = ?1", [key], |row| {
                row.get::<_, String>(0)
            })
            .ok()
        };
        (
            get("p2p.vault_key_encrypted"),
            get("p2p.vault_key_nonce"),
            get("p2p.device_id"),
        )
    };

    if let (Some(ct), Some(nonce_hex), Some(device_id)) = (vault_key_ct, vault_key_nonce, device_id)
    {
        let ct_bytes = base64::engine::general_purpose::STANDARD
            .decode(ct.as_bytes())
            .map_err(|_| AppError::InvalidInput("invalid vault key ciphertext".into()))?;
        let nonce_bytes = hex::decode(&nonce_hex)
            .map_err(|_| AppError::InvalidInput("invalid vault key nonce".into()))?;
        if nonce_bytes.len() != 12 {
            return Err(AppError::InvalidInput(
                "vault key nonce length mismatch".into(),
            ));
        }
        let mut nonce = [0u8; 12];
        nonce.copy_from_slice(&nonce_bytes);

        use sha2::{Digest, Sha256};
        let device_enc_key: [u8; 32] =
            Sha256::digest(format!("shellmate-auto-unlock-{device_id}").as_bytes()).into();
        let vault_key = crate::crypto::decrypt(
            &device_enc_key,
            &crate::crypto::EncryptedBlob {
                ciphertext: ct_bytes,
                nonce,
            },
        )?;
        if vault_key.len() != 32 {
            return Err(AppError::InvalidInput("vault key length mismatch".into()));
        }
        let mut vk = [0u8; 32];
        vk.copy_from_slice(&vault_key);
        app_state.vault.unlock_with_vault_key(&vk)?;
        log::info!("Auto-unlocked vault from stored satellite key");
        return Ok("Vault unlocked".into());
    }

    // Fallback: try to get vault key from desktop via P2P
    let (pairing_code, token, device_id, device_name) = {
        let conn = app_state.db.lock();
        let get = |key: &str| -> Option<String> {
            conn.query_row("SELECT value FROM settings WHERE key = ?1", [key], |row| {
                row.get::<_, String>(0)
            })
            .ok()
        };
        (
            get("p2p.desktop_pairing_code"),
            get("p2p.desktop_token"),
            get("p2p.device_id"),
            get("p2p.device_name").unwrap_or_else(|| "ShellMate Mobile".into()),
        )
    };

    let pairing_code =
        pairing_code.ok_or_else(|| AppError::InvalidInput("no paired desktop saved".into()))?;
    let token = token.ok_or_else(|| AppError::InvalidInput("no desktop token saved".into()))?;
    let device_id =
        device_id.ok_or_else(|| AppError::InvalidInput("no paired device id saved".into()))?;
    let code = decode_pairing_code(&pairing_code)?;
    let endpoints = endpoints_from_pairing_code(&code);

    let client = get_reqwest_client();
    let pin_for_request = code.pin.clone();
    let pull = PullRequest {
        device_id,
        device_name,
        pin: Some(pin_for_request),
        token: Some(token),
        wants_unlock: true,
    };

    let (pull_response, _endpoint) =
        request_desktop_pull(client, &endpoints, &pull, "auto-unlock").await?;

    import_vault_key_export(&app_state, &pull_response, &code)?;

    Ok("Vault unlocked via desktop".into())
}

#[tauri::command]
pub async fn p2p_sync_with_saved_desktop(app_state: State<'_, AppState>) -> AppResult<String> {
    p2p_sync_impl_inner(&app_state).await
}

#[tauri::command]
pub async fn p2p_auto_sync(app: AppHandle, app_state: State<'_, AppState>) -> AppResult<String> {
    match p2p_sync_impl_inner(&app_state).await {
        Ok(msg) => {
            let _ = app.emit("p2p:auto-sync-complete", &msg);
            Ok(msg)
        }
        Err(e) => {
            log::info!("Auto-sync skipped: {e}");
            Err(e)
        }
    }
}

async fn p2p_sync_impl_inner(app_state: &AppState) -> AppResult<String> {
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
        pin: Some(code.pin.clone()),
        token: Some(token),
        wants_unlock: false,
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

    // Decrypt the payload using the PIN from the saved pairing code.
    let payload: SyncPayload = if let Some(ref sk_enc) = pull_response.session_key_encrypted {
        let pin_salt = b"shellmate-p2p-pin-salt-v1";
        let pin_key = {
            let mut key = [0u8; 32];
            Argon2::default()
                .hash_password_into(code.pin.as_bytes(), pin_salt, &mut key)
                .map_err(|_| AppError::InvalidInput("PIN key derivation failed".into()))?;
            key
        };
        let enc_key = derive_encryption_key(&pin_key);
        let sk_bytes = base64::engine::general_purpose::STANDARD
            .decode(sk_enc.as_bytes())
            .map_err(|_| AppError::InvalidInput("invalid session key encoding".into()))?;
        let session_key = {
            let data = decrypt_payload(&enc_key, &sk_bytes)?;
            if data.len() != 32 {
                return Err(AppError::InvalidInput("session key length mismatch".into()));
            }
            let mut k = [0u8; 32];
            k.copy_from_slice(&data);
            k
        };

        let transfer_key = derive_encryption_key(&session_key);
        let encrypted_bytes = base64::engine::general_purpose::STANDARD
            .decode(pull_response.payload.as_bytes())
            .map_err(|_| AppError::InvalidInput("invalid encrypted payload".into()))?;
        let payload_json = decrypt_payload(&transfer_key, &encrypted_bytes)?;
        serde_json::from_slice(&payload_json)
            .map_err(|e| AppError::InvalidInput(format!("invalid sync payload: {e}")))?
    } else {
        let payload_bytes = base64::engine::general_purpose::STANDARD
            .decode(pull_response.payload.as_bytes())
            .map_err(|_| AppError::InvalidInput("invalid sync payload".into()))?;
        serde_json::from_slice(&payload_bytes)
            .map_err(|e| AppError::InvalidInput(format!("invalid sync payload: {e}")))?
    };

    import_vault_key_export(app_state, &pull_response, &code)?;
    merge_payload(&app_state.db, &app_state.vault, &payload)
}

#[allow(dead_code)]
pub async fn p2p_post_desktop_terminal(
    app_state: &AppState,
    path: &str,
    mut body: serde_json::Map<String, serde_json::Value>,
) -> AppResult<serde_json::Value> {
    let (pairing_code, token, device_id, device_name, saved_endpoint) = {
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
            get_setting("p2p.desktop_endpoint"),
        )
    };

    let pairing_code =
        pairing_code.ok_or_else(|| AppError::InvalidInput("no paired desktop saved".into()))?;
    let token = token.ok_or_else(|| AppError::InvalidInput("no desktop token saved".into()))?;
    let device_id =
        device_id.ok_or_else(|| AppError::InvalidInput("no paired device id saved".into()))?;
    let code = decode_pairing_code(&pairing_code)?;
    let mut endpoints = endpoints_from_pairing_code(&code);
    if let Some(endpoint) = saved_endpoint
        .as_deref()
        .and_then(endpoint_from_host_port)
        .filter(|saved| {
            !endpoints
                .iter()
                .any(|endpoint| endpoint.host == saved.host && endpoint.port == saved.port)
        })
    {
        endpoints.insert(0, endpoint);
    }

    body.insert("deviceId".into(), serde_json::json!(device_id));
    body.insert("deviceName".into(), serde_json::json!(device_name));
    body.insert("token".into(), serde_json::json!(token));

    let client = get_reqwest_client();

    let mut failures = Vec::new();
    for endpoint in endpoints {
        let proto = if endpoint.port == 443 { "https" } else { "http" };
        let url = format!("{}://{}:{}{}", proto, endpoint.host, endpoint.port, path);
        match client
            .post(&url)
            .timeout(std::time::Duration::from_secs(2))
            .json(&body)
            .send()
            .await
        {
            Ok(response) if response.status().is_success() => {
                let endpoint_value = format!("{}:{}", endpoint.host, endpoint.port);
                {
                    let conn = app_state.db.lock();
                    let _ = conn.execute(
                        "INSERT INTO settings (key, value) VALUES ('p2p.desktop_endpoint', ?1)
                         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
                        [&endpoint_value],
                    );
                }
                return response.json().await.map_err(|error| {
                    AppError::Internal(format!("invalid desktop terminal response: {error}"))
                });
            }
            Ok(response) => {
                let status = response.status();
                let text = response.text().await.unwrap_or_default();
                failures.push(format!(
                    "{} {}:{} -> HTTP {status}: {text}",
                    endpoint.label, endpoint.host, endpoint.port
                ));
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

#[allow(dead_code)]
pub fn start_desktop_terminal_stream(
    app: tauri::AppHandle,
    db: Arc<Mutex<Connection>>,
    local_session_output: Arc<DashMap<String, tokio::sync::Mutex<String>>>,
    session_id: String,
) {
    use futures::StreamExt;

    tokio::spawn(async move {
        let clean_session_id = session_id.trim_start_matches("desktop:").to_string();
        log::info!("Starting persistent desktop terminal stream for {}", clean_session_id);

        let mut retry_delay = std::time::Duration::from_secs(1);
        loop {
            let (pairing_code, token, device_id, device_name, saved_endpoint) = {
                let conn = db.lock();
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
                    get_setting("p2p.desktop_endpoint"),
                )
            };

            let (pairing_code, token, device_id) = match (pairing_code, token, device_id) {
                (Some(c), Some(t), Some(d)) => (c, t, d),
                _ => {
                    log::warn!("Missing pairing settings for stream connection, retrying in 5s");
                    tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                    continue;
                }
            };

            let code = match decode_pairing_code(&pairing_code) {
                Ok(c) => c,
                Err(_) => {
                    tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                    continue;
                }
            };

            let mut endpoints = endpoints_from_pairing_code(&code);
            if let Some(endpoint) = saved_endpoint
                .as_deref()
                .and_then(endpoint_from_host_port)
                .filter(|saved| {
                    !endpoints
                        .iter()
                        .any(|endpoint| endpoint.host == saved.host && endpoint.port == saved.port)
                })
            {
                endpoints.insert(0, endpoint);
            }

            let mut body = serde_json::Map::new();
            body.insert("deviceId".into(), serde_json::json!(device_id));
            body.insert("deviceName".into(), serde_json::json!(device_name));
            body.insert("token".into(), serde_json::json!(token));
            body.insert("sessionId".into(), serde_json::json!(clean_session_id));

            let client = get_reqwest_client();
            let mut connected = false;

            for endpoint in endpoints {
                let proto = if endpoint.port == 443 { "https" } else { "http" };
                let url = format!("{}://{}:{}/terminal/stream", proto, endpoint.host, endpoint.port);

                log::info!("Connecting to desktop terminal stream at {}", url);
                let mut response = match client
                    .post(&url)
                    .json(&body)
                    .send()
                    .await
                {
                    Ok(res) if res.status().is_success() => res,
                    Ok(res) => {
                        log::warn!("Stream endpoint {} returned status: {}", url, res.status());
                        continue;
                    }
                    Err(e) => {
                        log::warn!("Failed to connect to stream endpoint {}: {}", url, e);
                        continue;
                    }
                };

                log::info!("Desktop terminal stream connected successfully to {}", url);
                retry_delay = std::time::Duration::from_secs(1);
                connected = true;

                // Notify frontend that the session is connected
                let _ = app.emit(
                    &format!("ssh:status:desktop:{}", clean_session_id),
                    serde_json::json!({
                        "sessionId": format!("desktop:{}", clean_session_id),
                        "status": "connected",
                        "message": null,
                    }),
                );

                let mut line_buffer = Vec::new();

                loop {
                    match response.chunk().await {
                        Ok(Some(chunk)) => {
                            line_buffer.extend_from_slice(&chunk);

                            while let Some(pos) = line_buffer.iter().position(|&b| b == b'\n') {
                                let line_bytes = line_buffer.drain(..=pos).collect::<Vec<u8>>();
                                let line = String::from_utf8_lossy(&line_bytes);
                                let trimmed = line.trim();
                                if trimmed.starts_with("data: ") {
                                    let json_str = &trimmed[6..];
                                    if let Ok(val) = serde_json::from_str::<serde_json::Value>(json_str) {
                                        let msg_type = val.get("type").and_then(|t| t.as_str()).unwrap_or_default();
                                        match msg_type {
                                            "output" => {
                                                if let Some(data) = val.get("data").and_then(|d| d.as_str()) {
                                                    // 1. Emit direct push event to frontend
                                                    let _ = app.emit(
                                                        &format!("ssh:output:desktop:{}", clean_session_id),
                                                        serde_json::json!({
                                                            "sessionId": format!("desktop:{}", clean_session_id),
                                                            "data": data,
                                                        }),
                                                    );

                                                    // 2. Append to local output buffer for fallback
                                                    let output_key = format!("desktop:{}", clean_session_id);
                                                    if let Some(entry) = local_session_output.get(&output_key) {
                                                        if let Ok(mut buffer) = entry.value().try_lock() {
                                                            buffer.push_str(data);
                                                        }
                                                    }
                                                }
                                            }
                                            "status" => {
                                                let status = val.get("status").and_then(|s| s.as_str()).unwrap_or_default();
                                                let message = val.get("message").and_then(|m| m.as_str());
                                                let _ = app.emit(
                                                    &format!("ssh:status:desktop:{}", clean_session_id),
                                                    serde_json::json!({
                                                        "sessionId": format!("desktop:{}", clean_session_id),
                                                        "status": status,
                                                        "message": message,
                                                    }),
                                                );
                                            }
                                            "error" => {
                                                let message = val.get("message").and_then(|m| m.as_str()).unwrap_or_default();
                                                let _ = app.emit(
                                                    &format!("ssh:error:desktop:{}", clean_session_id),
                                                    serde_json::json!({
                                                        "sessionId": format!("desktop:{}", clean_session_id),
                                                        "message": message,
                                                    }),
                                                );
                                            }
                                            _ => {}
                                        }
                                    }
                                }
                            }
                        }
                        Ok(None) => {
                            log::info!("Desktop terminal stream ended (EOF)");
                            break;
                        }
                        Err(e) => {
                            log::warn!("Error reading stream chunk: {}", e);
                            break;
                        }
                    }
                }

                log::warn!("Desktop terminal stream disconnected from {}", url);
                let _ = app.emit(
                    &format!("ssh:status:desktop:{}", clean_session_id),
                    serde_json::json!({
                        "sessionId": format!("desktop:{}", clean_session_id),
                        "status": "connecting",
                        "message": "Reconnecting...",
                    }),
                );
                break;
            }

            if !connected {
                log::warn!("Could not connect to any desktop terminal stream endpoint. Retrying in {:?}", retry_delay);
                tokio::time::sleep(retry_delay).await;
                retry_delay = std::cmp::min(retry_delay * 2, std::time::Duration::from_secs(30));
            }
        }
    });
}

#[tauri::command]
pub async fn p2p_list_remote_files(
    state: State<'_, AppState>,
    path: Option<String>,
) -> AppResult<serde_json::Value> {
    #[cfg(any(target_os = "android", target_os = "ios"))]
    {
        Err(crate::errors::AppError::Internal("Not supported on mobile".into()))
    }
    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    {
        let mut body = serde_json::Map::new();
        body.insert("path".into(), serde_json::json!(path));
        p2p_post_desktop_terminal(&state, "/files/list", body).await
    }
}

#[tauri::command]
pub async fn p2p_delete_remote_file(
    state: State<'_, AppState>,
    path: String,
) -> AppResult<serde_json::Value> {
    #[cfg(any(target_os = "android", target_os = "ios"))]
    {
        Err(crate::errors::AppError::Internal("Not supported on mobile".into()))
    }
    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    {
        let mut body = serde_json::Map::new();
        body.insert("path".into(), serde_json::json!(path));
        p2p_post_desktop_terminal(&state, "/files/delete", body).await
    }
}

#[tauri::command]
pub async fn p2p_send_remote_desktop_input(
    state: State<'_, AppState>,
    event: String,
    x: Option<i32>,
    y: Option<i32>,
    button: Option<String>,
    key: Option<String>,
) -> AppResult<serde_json::Value> {
    #[cfg(any(target_os = "android", target_os = "ios"))]
    {
        Err(crate::errors::AppError::Internal("Not supported on mobile".into()))
    }
    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    {
        let mut body = serde_json::Map::new();
        body.insert("event".into(), serde_json::json!(event));
        body.insert("x".into(), serde_json::json!(x));
        body.insert("y".into(), serde_json::json!(y));
        body.insert("button".into(), serde_json::json!(button));
        body.insert("key".into(), serde_json::json!(key));
        p2p_post_desktop_terminal(&state, "/desktop/input", body).await
    }
}

#[tauri::command]
pub async fn p2p_get_remote_desktop_screenshot(
    state: State<'_, AppState>,
) -> AppResult<String> {
    #[cfg(any(target_os = "android", target_os = "ios"))]
    {
        Err(crate::errors::AppError::Internal("Not supported on mobile".into()))
    }
    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    {
        let (pairing_code, token, device_id, device_name, saved_endpoint) = {
            let conn = state.db.lock();
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
                get_setting("p2p.desktop_endpoint"),
            )
        };

        let pairing_code =
            pairing_code.ok_or_else(|| AppError::InvalidInput("no paired desktop saved".into()))?;
        let token = token.ok_or_else(|| AppError::InvalidInput("no desktop token saved".into()))?;
        let device_id =
            device_id.ok_or_else(|| AppError::InvalidInput("no paired device id saved".into()))?;
        let code = decode_pairing_code(&pairing_code)?;
        let mut endpoints = endpoints_from_pairing_code(&code);
        if let Some(endpoint) = saved_endpoint
            .as_deref()
            .and_then(endpoint_from_host_port)
            .filter(|saved| {
                !endpoints
                    .iter()
                    .any(|endpoint| endpoint.host == saved.host && endpoint.port == saved.port)
            })
        {
            endpoints.insert(0, endpoint);
        }

        let mut body = serde_json::Map::new();
        body.insert("deviceId".into(), serde_json::json!(device_id));
        body.insert("deviceName".into(), serde_json::json!(device_name));
        body.insert("token".into(), serde_json::json!(token));

        let client = get_reqwest_client();
        let mut failures = Vec::new();

        for endpoint in endpoints {
            let proto = if endpoint.port == 443 { "https" } else { "http" };
            let url = format!("{}://{}:{}/desktop/screenshot", proto, endpoint.host, endpoint.port);
            match client
                .post(&url)
                .timeout(std::time::Duration::from_secs(3))
                .json(&body)
                .send()
                .await
            {
                Ok(response) if response.status().is_success() => {
                    let endpoint_value = format!("{}:{}", endpoint.host, endpoint.port);
                    {
                        let conn = state.db.lock();
                        let _ = conn.execute(
                            "INSERT INTO settings (key, value) VALUES ('p2p.desktop_endpoint', ?1)
                             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
                            [&endpoint_value],
                        );
                    }
                    let bytes = response.bytes().await.map_err(|e| {
                        AppError::Internal(format!("failed to read screenshot bytes: {e}"))
                    })?;
                    use base64::Engine;
                    return Ok(base64::engine::general_purpose::STANDARD.encode(&bytes));
                }
                Ok(response) => {
                    let status = response.status();
                    failures.push(format!("{}:{} -> HTTP {status}", endpoint.host, endpoint.port));
                }
                Err(error) => {
                    failures.push(format!("{}:{} -> {error}", endpoint.host, endpoint.port));
                }
            }
        }

        Err(AppError::Internal(format!(
            "failed to fetch screenshot: ({})",
            failures.join("; ")
        )))
    }
}

#[tauri::command]
pub async fn p2p_download_remote_file(
    state: State<'_, AppState>,
    remote_path: String,
    local_path: String,
) -> AppResult<()> {
    #[cfg(any(target_os = "android", target_os = "ios"))]
    {
        Err(crate::errors::AppError::Internal("Not supported on mobile".into()))
    }
    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    {
        let (pairing_code, token, device_id, device_name, saved_endpoint) = {
            let conn = state.db.lock();
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
                get_setting("p2p.desktop_endpoint"),
            )
        };

        let pairing_code =
            pairing_code.ok_or_else(|| AppError::InvalidInput("no paired desktop saved".into()))?;
        let token = token.ok_or_else(|| AppError::InvalidInput("no desktop token saved".into()))?;
        let device_id =
            device_id.ok_or_else(|| AppError::InvalidInput("no paired device id saved".into()))?;
        let code = decode_pairing_code(&pairing_code)?;
        let mut endpoints = endpoints_from_pairing_code(&code);
        if let Some(endpoint) = saved_endpoint
            .as_deref()
            .and_then(endpoint_from_host_port)
            .filter(|saved| {
                !endpoints
                    .iter()
                    .any(|endpoint| endpoint.host == saved.host && endpoint.port == saved.port)
            })
        {
            endpoints.insert(0, endpoint);
        }

        let mut body = serde_json::Map::new();
        body.insert("deviceId".into(), serde_json::json!(device_id));
        body.insert("deviceName".into(), serde_json::json!(device_name));
        body.insert("token".into(), serde_json::json!(token));
        body.insert("path".into(), serde_json::json!(remote_path));

        let client = get_reqwest_client();
        let mut failures = Vec::new();

        for endpoint in endpoints {
            let proto = if endpoint.port == 443 { "https" } else { "http" };
            let url = format!("{}://{}:{}/files/download", proto, endpoint.host, endpoint.port);
            match client
                .post(&url)
                .timeout(std::time::Duration::from_secs(30))
                .json(&body)
                .send()
                .await
            {
                Ok(response) if response.status().is_success() => {
                    let mut file = tokio::fs::File::create(&local_path).await.map_err(|e| {
                        AppError::Internal(format!("failed to create local file: {e}"))
                    })?;
                    let mut response = response;
                    while let Ok(Some(chunk)) = response.chunk().await {
                        tokio::io::AsyncWriteExt::write_all(&mut file, &chunk).await.map_err(|e| {
                            AppError::Internal(format!("failed to write to local file: {e}"))
                        })?;
                    }
                    let _ = tokio::io::AsyncWriteExt::flush(&mut file).await;
                    return Ok(());
                }
                Ok(response) => {
                    let status = response.status();
                    failures.push(format!("{}:{} -> HTTP {status}", endpoint.host, endpoint.port));
                }
                Err(error) => {
                    failures.push(format!("{}:{} -> {error}", endpoint.host, endpoint.port));
                }
            }
        }

        Err(AppError::Internal(format!(
            "failed to download file: ({})",
            failures.join("; ")
        )))
    }
}

#[tauri::command]
pub async fn p2p_upload_remote_file(
    state: State<'_, AppState>,
    remote_path: String,
    local_path: String,
) -> AppResult<()> {
    #[cfg(any(target_os = "android", target_os = "ios"))]
    {
        Err(crate::errors::AppError::Internal("Not supported on mobile".into()))
    }
    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    {
        let (pairing_code, token, device_id, device_name, saved_endpoint) = {
            let conn = state.db.lock();
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
                get_setting("p2p.desktop_endpoint"),
            )
        };

        let pairing_code =
            pairing_code.ok_or_else(|| AppError::InvalidInput("no paired desktop saved".into()))?;
        let token = token.ok_or_else(|| AppError::InvalidInput("no desktop token saved".into()))?;
        let device_id =
            device_id.ok_or_else(|| AppError::InvalidInput("no paired device id saved".into()))?;
        let code = decode_pairing_code(&pairing_code)?;
        let mut endpoints = endpoints_from_pairing_code(&code);
        if let Some(endpoint) = saved_endpoint
            .as_deref()
            .and_then(endpoint_from_host_port)
            .filter(|saved| {
                !endpoints
                    .iter()
                    .any(|endpoint| endpoint.host == saved.host && endpoint.port == saved.port)
            })
        {
            endpoints.insert(0, endpoint);
        }

        let bytes = tokio::fs::read(&local_path).await.map_err(|e| {
            AppError::InvalidInput(format!("failed to read local file: {e}"))
        })?;

        let client = get_reqwest_client();
        let mut failures = Vec::new();

        for endpoint in endpoints {
            let proto = if endpoint.port == 443 { "https" } else { "http" };
            let url = format!("{}://{}:{}/files/upload", proto, endpoint.host, endpoint.port);
            match client
                .post(&url)
                .header("X-File-Path", &remote_path)
                .header("X-Device-Id", &device_id)
                .header("X-Device-Name", &device_name)
                .header("X-Token", &token)
                .header("Content-Length", bytes.len())
                .body(bytes.clone())
                .send()
                .await
            {
                Ok(response) if response.status().is_success() => {
                    return Ok(());
                }
                Ok(response) => {
                    let status = response.status();
                    failures.push(format!("{}:{} -> HTTP {status}", endpoint.host, endpoint.port));
                }
                Err(error) => {
                    failures.push(format!("{}:{} -> {error}", endpoint.host, endpoint.port));
                }
            }
        }

        Err(AppError::Internal(format!(
            "failed to upload file: ({})",
            failures.join("; ")
        )))
    }
}
