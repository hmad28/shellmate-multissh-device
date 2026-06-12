use crate::errors::{AppError, AppResult};
use crate::vault::Vault;
use aes_gcm::{aead::Aead, Aes256Gcm, KeyInit, Nonce};
use base64::Engine;
use parking_lot::Mutex;
use rand::Rng;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::{AppHandle, Emitter, State};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::sync::oneshot;

pub struct SyncServerState {
    inner: Mutex<SyncServerInner>,
}

struct SyncServerInner {
    shutdown_tx: Option<oneshot::Sender<()>>,
    pin: Option<String>,
    is_running: bool,
}

impl SyncServerState {
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(SyncServerInner {
                shutdown_tx: None,
                pin: None,
                is_running: false,
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
    payload: Vec<u8>,
}

fn generate_pin() -> String {
    let mut rng = rand::thread_rng();
    format!("{:06}", rng.gen_range(0..1_000_000))
}

fn derive_key_from_pin(pin: &str) -> [u8; 32] {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(b"shellmate-sync-v1");
    hasher.update(pin.as_bytes());
    let result = hasher.finalize();
    let mut key = [0u8; 32];
    key.copy_from_slice(&result);
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
        let response = "HTTP/1.1 401 Unauthorized\r\nContent-Length: 0\r\n\r\n";
        let _ = stream.write_all(response.as_bytes()).await;
        return;
    }

    let key = derive_key_from_pin(&req.pin);
    let decrypted = match decrypt_payload(&key, &req.payload) {
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
            ("200 OK", serde_json::json!({"success": true, "message": msg}))
        }
        Err(e) => {
            let msg = format!("Merge failed: {e}");
            ("500 Internal Server Error", serde_json::json!({"success": false, "message": msg}))
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

fn merge_payload(
    db: &Arc<Mutex<Connection>>,
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

    Ok(format!("Imported {imported} items, skipped {skipped} duplicates"))
}

async fn run_server(
    app: AppHandle,
    db: Arc<Mutex<Connection>>,
    vault: Arc<Vault>,
    server_state: Arc<SyncServerState>,
    mut shutdown_rx: oneshot::Receiver<()>,
) {
    let listener = TcpListener::bind("0.0.0.0:0").await;
    let listener = match listener {
        Ok(l) => l,
        Err(e) => {
            log::error!("Failed to bind sync server: {e}");
            return;
        }
    };

    let port = listener.local_addr().map(|a| a.port()).unwrap_or(0);
    log::info!("P2P sync server listening on port {port}");

    let pin = {
        let mut inner = server_state.inner.lock();
        inner.pin = Some(generate_pin());
        inner.pin.clone().unwrap()
    };

    let _ = app.emit("p2p:pairing-ready", &pin);

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
                        let pin = pin.clone();

                        tokio::spawn(async move {
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
                                    let body = serde_json::json!({
                                        "pin": &pin,
                                        "port": port,
                                    });
                                    let response = format!(
                                        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
                                        body.to_string().len(),
                                        body
                                    );
                                    let _ = stream.write_all(response.as_bytes()).await;
                                }
                                ("POST", "/sync") => {
                                    handle_sync_receive(
                                        &mut stream,
                                        &buf,
                                        &pin,
                                        &db,
                                        &vault,
                                        &app,
                                    ).await;
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
    inner.shutdown_tx = None;
}

#[tauri::command]
pub async fn p2p_start_sync_server(
    app: AppHandle,
    state: State<'_, Arc<SyncServerState>>,
    db: State<'_, Arc<Mutex<Connection>>>,
    vault: State<'_, Arc<Vault>>,
) -> AppResult<String> {
    {
        let inner = state.inner.lock();
        if inner.is_running {
            return Err(AppError::InvalidInput("sync server already running".into()));
        }
    }

    let (shutdown_tx, shutdown_rx) = oneshot::channel();

    {
        let mut inner = state.inner.lock();
        inner.is_running = true;
        inner.shutdown_tx = Some(shutdown_tx);
    }

    let server_state = Arc::clone(&state);
    let db = Arc::clone(&db);
    let vault = Arc::clone(&vault);

    tokio::spawn(run_server(app, db, vault, server_state, shutdown_rx));

    let pin = {
        let inner = state.inner.lock();
        inner.pin.clone().unwrap_or_default()
    };

    Ok(pin)
}

#[tauri::command]
pub async fn p2p_stop_sync_server(
    state: State<'_, Arc<SyncServerState>>,
) -> AppResult<()> {
    let mut inner = state.inner.lock();
    if let Some(tx) = inner.shutdown_tx.take() {
        let _ = tx.send(());
    }
    inner.is_running = false;
    inner.pin = None;
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
    }))
}

#[tauri::command]
pub async fn p2p_export_for_sync(
    db: State<'_, Arc<Mutex<Connection>>>,
) -> AppResult<String> {
    let conn = db.lock();

    let hosts: Vec<HostExport> = {
        let mut stmt = conn.prepare(
            "SELECT id, label, hostname, port, username, auth_type, credential_id, group_id, tags, notes, created_at, updated_at
             FROM hosts",
        )?;
        let rows: Vec<_> = stmt.query_map([], |row| {
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
        })?.collect::<Result<Vec<_>, _>>()?;
        rows
    };

    let credentials: Vec<CredentialExport> = {
        let mut stmt = conn.prepare(
            "SELECT id, type, encrypted_data, nonce, created_at, updated_at FROM credentials",
        )?;
        let rows: Vec<_> = stmt.query_map([], |row| {
            Ok(CredentialExport {
                id: row.get(0)?,
                cred_type: row.get(1)?,
                encrypted_data: row.get(2)?,
                nonce: row.get(3)?,
                created_at: row.get(4)?,
                updated_at: row.get(5)?,
            })
        })?.collect::<Result<Vec<_>, _>>()?;
        rows
    };

    let groups: Vec<GroupExport> = {
        let mut stmt = conn.prepare("SELECT id, name, color, parent_id, sort_order FROM groups")?;
        let rows: Vec<_> = stmt.query_map([], |row| {
            Ok(GroupExport {
                id: row.get(0)?,
                name: row.get(1)?,
                color: row.get(2)?,
                parent_id: row.get(3)?,
                sort_order: row.get(4)?,
            })
        })?.collect::<Result<Vec<_>, _>>()?;
        rows
    };

    let snippets: Vec<SnippetExport> = {
        let mut stmt = conn.prepare(
            "SELECT id, title, command, description, tags, created_at, updated_at FROM snippets",
        )?;
        let rows: Vec<_> = stmt.query_map([], |row| {
            Ok(SnippetExport {
                id: row.get(0)?,
                title: row.get(1)?,
                command: row.get(2)?,
                description: row.get(3)?,
                tags: row.get(4)?,
                created_at: row.get(5)?,
                updated_at: row.get(6)?,
            })
        })?.collect::<Result<Vec<_>, _>>()?;
        rows
    };

    let payload = SyncPayload {
        hosts,
        credentials,
        groups,
        snippets,
    };

    let json = serde_json::to_vec(&payload)
        .map_err(|e| AppError::Internal(format!("serialization failed: {e}")))?;

    Ok(base64::engine::general_purpose::STANDARD.encode(&json))
}
