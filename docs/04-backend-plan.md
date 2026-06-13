# Backend Plan
## ShellMate — Rust + Tauri Backend (v1.0 Production)

**Version:** 2.3
**Last Updated:** 2026-06-13

---

## 1. Backend Architecture

### 1.1 Technology Stack
| Technology | Version | Purpose |
|------------|---------|---------|
| Rust | Latest stable | Core language |
| Tauri | v2.x | App framework |
| russh | 0.45.x | SSH implementation |
| rusqlite | 0.32.x | SQLite bindings |
| argon2 | 0.5.x | Key derivation |
| aes-gcm | 0.10.x | Encryption |
| uuid | 1.x | ID generation |
| serde | 1.x | Serialization |

### 1.2 Module Structure
```
src-tauri/src/
├── main.rs              # Entry point
├── lib.rs               # Library exports
│
├── commands/            # Tauri command handlers
│   ├── mod.rs
│   ├── host.rs          # Host CRUD
│   ├── group.rs         # Group CRUD + move_host_to_group
│   ├── ssh.rs           # SSH connect/disconnect/send/resize
│   ├── vault.rs         # Vault setup/unlock/lock/check_idle/change_master_password
│   ├── credential.rs    # Save/delete credentials (vault-gated)
│   ├── snippet.rs       # Snippet CRUD
│   ├── sftp.rs          # SFTP open/list/upload/download/rename/remove/mkdir/close
│   ├── port_forward.rs  # Port forward create/list/remove/toggle
│   ├── settings.rs      # Settings get/set
│   ├── theme.rs         # Theme CRUD (builtin protection)
│   ├── known_hosts.rs   # TOFU verify/trust/list/remove
│   ├── broadcast.rs     # Broadcast add/remove/send
│   ├── discovery.rs     # mDNS discovery + broadcasting
│   └── system.rs        # app_version
│
├── ssh/                 # SSH implementation
│   ├── mod.rs
│   ├── handler.rs       # russh client handler (TOFU host key, auth)
│   ├── session.rs       # Session management (1 conn per tab)
│   ├── reconnect.rs     # Auto-reconnect with exponential backoff
│   └── broadcast.rs     # Multi-tab keystroke broadcasting
│
├── sftp/                # SFTP implementation (russh-sftp)
│   └── mod.rs
│
├── db/                  # Database layer (SQLite)
│   ├── mod.rs           # Schema, migrations, queries
│   └── (inline query functions — no separate queries/ directory)
│
├── crypto/              # Encryption
│   ├── mod.rs
│   ├── aes.rs           # AES-256-GCM
│   ├── kdf.rs           # Argon2id key derivation
│   └── secure_buffer.rs # Zeroize-on-drop secret wrapper
│
├── errors.rs            # Error types
├── state.rs             # App state
└── utils.rs             # Utilities
```

---

## 2. Tauri Commands

### 2.1 Host Commands (host.rs)
```rust
// CRUD operations for SSH hosts

#[tauri::command]
async fn create_host(state: State<'_, AppState>, host: HostInput) -> Result<Host, AppError> {
    // Validate input
    // Insert into database
    // Return created host
}

#[tauri::command]
async fn get_hosts(state: State<'_, AppState>) -> Result<Vec<Host>, AppError> {
    // Query all hosts
    // Return list
}

#[tauri::command]
async fn update_host(state: State<'_, AppState>, id: String, host: HostInput) -> Result<Host, AppError> {
    // Validate input
    // Update database
    // Return updated host
}

#[tauri::command]
async fn delete_host(state: State<'_, AppState>, id: String) -> Result<(), AppError> {
    // Delete from database
    // Cleanup related data
}

#[tauri::command]
async fn search_hosts(state: State<'_, AppState>, query: String) -> Result<Vec<Host>, AppError> {
    // Search by label/hostname
    // Return matching hosts
}
```

### 2.2 SSH Commands (ssh.rs)
```rust
// SSH connection and session management

#[tauri::command]
async fn ssh_connect(
    state: State<'_, AppState>,
    host_id: String,
) -> Result<String, AppError> {
    // Get host from database
    // Get credentials from vault
    // Establish SSH connection
    // Create session
    // Return session ID
}

#[tauri::command]
async fn ssh_disconnect(
    state: State<'_, AppState>,
    session_id: String,
) -> Result<(), AppError> {
    // Close SSH session
    // Cleanup resources
}

#[tauri::command]
async fn ssh_send(
    state: State<'_, AppState>,
    session_id: String,
    data: String,
) -> Result<(), AppError> {
    // Send data to SSH session
}

// Events for streaming
// - ssh_output: Stream terminal output to frontend
// - ssh_error: Stream errors to frontend
// - ssh_status: Connection status updates
```

### 2.3 Vault Commands (vault.rs)
```rust
// Credential encryption and vault management

#[tauri::command]
async fn vault_setup(
    state: State<'_, AppState>,
    master_password: String,
) -> Result<(), AppError> {
    // Derive key from master password
    // Initialize vault
    // Store encrypted master key hash
}

#[tauri::command]
async fn vault_unlock(
    state: State<'_, AppState>,
    master_password: String,
) -> Result<bool, AppError> {
    // Verify master password
    // Derive encryption key
    // Unlock vault
    // Return success
}

#[tauri::command]
async fn vault_lock(state: State<'_, AppState>) -> Result<(), AppError> {
    // Clear encryption key from memory
    // Lock vault
}

#[tauri::command]
async fn save_credential(
    state: State<'_, AppState>,
    data: String,
) -> Result<String, AppError> {
    // Encrypt data with vault key
    // Store in database
    // Return credential ID
}

#[tauri::command]
async fn get_credential(
    state: State<'_, AppState>,
    credential_id: String,
) -> Result<String, AppError> {
    // Get encrypted data from database
    // Decrypt with vault key
    // Return plaintext
}
```

### 2.4 SFTP Commands (sftp.rs)
```rust
// SFTP file operations

#[tauri::command]
async fn sftp_list(
    state: State<'_, AppState>,
    session_id: String,
    path: String,
) -> Result<Vec<SftpEntry>, AppError> {
    // List directory contents
    // Return file/folder entries
}

#[tauri::command]
async fn sftp_upload(
    state: State<'_, AppState>,
    session_id: String,
    local_path: String,
    remote_path: String,
) -> Result<(), AppError> {
    // Upload file to remote
    // Emit progress events
}

#[tauri::command]
async fn sftp_download(
    state: State<'_, AppState>,
    session_id: String,
    remote_path: String,
    local_path: String,
) -> Result<(), AppError> {
    // Download file from remote
    // Emit progress events
}

#[tauri::command]
async fn sftp_mkdir(
    state: State<'_, AppState>,
    session_id: String,
    path: String,
) -> Result<(), AppError> {
    // Create remote directory
}

#[tauri::command]
async fn sftp_delete(
    state: State<'_, AppState>,
    session_id: String,
    path: String,
) -> Result<(), AppError> {
    // Delete remote file/directory
}

#[tauri::command]
async fn sftp_rename(
    state: State<'_, AppState>,
    session_id: String,
    old_path: String,
    new_path: String,
) -> Result<(), AppError> {
    // Rename remote file/directory
}
```

---

## 3. SSH Implementation

### 3.1 Connection Handler (connection.rs)
```rust
use russh::*;
use russh_keys::*;

pub struct SshConnection {
    session: Option<Session>,
    host: String,
    port: u16,
    username: String,
}

impl SshConnection {
    pub async fn connect(
        host: &str,
        port: u16,
        username: &str,
        auth: AuthMethod,
    ) -> Result<Self, SshError> {
        // Create SSH client config
        // Connect to server
        // Authenticate
        // Return connection
    }
    
    pub async fn execute(&mut self, command: &str) -> Result<String, SshError> {
        // Execute command on remote
        // Return output
    }
    
    pub async fn open_shell(&mut self) -> Result<Channel, SshError> {
        // Open interactive shell
        // Return channel for I/O
    }
    
    pub async fn close(&mut self) -> Result<(), SshError> {
        // Close connection
        // Cleanup
    }
}
```

### 3.2 Session Manager (session.rs)
```rust
pub struct SessionManager {
    sessions: HashMap<String, SshSession>,
}

pub struct SshSession {
    id: String,
    host_id: String,
    connection: SshConnection,
    channel: Option<Channel>,
    status: ConnectionStatus,
}

impl SessionManager {
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
        }
    }
    
    pub async fn create_session(
        &mut self,
        host_id: String,
        host: Host,
        credentials: Credentials,
    ) -> Result<String, AppError> {
        // Create new SSH session
        // Store in hashmap
        // Return session ID
    }
    
    pub async fn send_data(
        &mut self,
        session_id: &str,
        data: &[u8],
    ) -> Result<(), AppError> {
        // Send data to session
    }
    
    pub async fn receive_data(
        &mut self,
        session_id: &str,
    ) -> Result<Vec<u8>, AppError> {
        // Receive data from session
    }
    
    pub async fn close_session(
        &mut self,
        session_id: &str,
    ) -> Result<(), AppError> {
        // Close session
        // Remove from hashmap
    }
}
```

### 3.3 Authentication (auth.rs)
```rust
pub enum AuthMethod {
    Password(String),
    Key {
        private_key_path: String,
        passphrase: Option<String>,
    },
}

impl AuthMethod {
    pub async fn authenticate(
        &self,
        username: &str,
    ) -> Result<russh_keys::key::KeyPair, AuthError> {
        match self {
            AuthMethod::Password(password) => {
                // Password authentication
            }
            AuthMethod::Key { private_key_path, passphrase } => {
                // Key authentication
            }
        }
    }
}
```

---

## 4. Database Layer

### 4.1 Schema (schema.rs)
```rust
pub const SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS hosts (
    id TEXT PRIMARY KEY,
    label TEXT NOT NULL,
    hostname TEXT NOT NULL,
    port INTEGER NOT NULL DEFAULT 22,
    username TEXT NOT NULL,
    auth_type TEXT NOT NULL CHECK (auth_type IN ('password', 'key', 'key_passphrase')),
    credential_id TEXT NOT NULL REFERENCES credentials(id),
    group_id TEXT REFERENCES groups(id),
    tags TEXT,
    notes TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS groups (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    color TEXT,
    parent_id TEXT REFERENCES groups(id),
    sort_order INTEGER DEFAULT 0
);

CREATE TABLE IF NOT EXISTS credentials (
    id TEXT PRIMARY KEY,
    type TEXT NOT NULL CHECK (type IN ('password', 'private_key')),
    encrypted_data BLOB NOT NULL,
    nonce BLOB NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS snippets (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    command TEXT NOT NULL,
    description TEXT,
    tags TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS port_forwards (
    id TEXT PRIMARY KEY,
    host_id TEXT NOT NULL REFERENCES hosts(id),
    type TEXT NOT NULL CHECK (type IN ('local', 'remote')),
    local_port INTEGER NOT NULL,
    remote_host TEXT NOT NULL,
    remote_port INTEGER NOT NULL,
    enabled INTEGER NOT NULL DEFAULT 1
);

CREATE TABLE IF NOT EXISTS settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);
"#;
```

### 4.2 Query Examples (queries/hosts.rs)
```rust
use rusqlite::{params, Connection};

pub fn create_host(conn: &Connection, host: &Host) -> Result<(), rusqlite::Error> {
    conn.execute(
        "INSERT INTO hosts (id, label, hostname, port, username, auth_type, credential_id, group_id, tags, notes, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
        params![
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
            host.updated_at,
        ],
    )?;
    Ok(())
}

pub fn get_hosts(conn: &Connection) -> Result<Vec<Host>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT id, label, hostname, port, username, auth_type, credential_id, group_id, tags, notes, created_at, updated_at
         FROM hosts ORDER BY label"
    )?;
    
    let hosts = stmt.query_map([], |row| {
        Ok(Host {
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
    
    Ok(hosts)
}
```

---

## 5. Encryption Module

### 5.1 Key Derivation (key_derivation.rs)
```rust
use argon2::{Argon2, Version, Algorithm};
use rand::Rng;

pub fn derive_key(
    master_password: &str,
    salt: &[u8],
) -> Result<[u8; 32], CryptoError> {
    let argon2 = Argon2::new(
        Algorithm::Argon2id,
        Version::Version13,
        argon2::Params::new(
            65536,  // Memory cost (64MB)
            3,      // Time cost
            4,      // Parallelism
            32,     // Output length
        ).map_err(|_| CryptoError::InvalidParams)?
    );
    
    let mut key = [0u8; 32];
    argon2
        .hash_password_into(master_password.as_bytes(), salt, &mut key)
        .map_err(|_| CryptoError::HashFailed)?;
    
    Ok(key)
}

pub fn generate_salt() -> [u8; 16] {
    let mut salt = [0u8; 16];
    rand::thread_rng().fill(&mut salt);
    salt
}
```

### 5.2 AES Encryption (aes.rs)
```rust
use aes_gcm::{Aes256Gcm, KeyInit, Nonce};
use aes_gcm::aead::Aead;
use rand::Rng;

pub struct AesEncryptor {
    cipher: Aes256Gcm,
}

impl AesEncryptor {
    pub fn new(key: &[u8; 32]) -> Self {
        let cipher = Aes256Gcm::new_from_slice(key)
            .expect("Invalid key length");
        Self { cipher }
    }
    
    pub fn encrypt(&self, plaintext: &[u8]) -> Result<(Vec<u8>, [u8; 12]), CryptoError> {
        let mut nonce_bytes = [0u8; 12];
        rand::thread_rng().fill(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);
        
        let ciphertext = self.cipher
            .encrypt(nonce, plaintext)
            .map_err(|_| CryptoError::EncryptionFailed)?;
        
        Ok((ciphertext, nonce_bytes))
    }
    
    pub fn decrypt(&self, ciphertext: &[u8], nonce: &[u8; 12]) -> Result<Vec<u8>, CryptoError> {
        let nonce = Nonce::from_slice(nonce);
        
        let plaintext = self.cipher
            .decrypt(nonce, ciphertext)
            .map_err(|_| CryptoError::DecryptionFailed)?;
        
        Ok(plaintext)
    }
}
```

---

## 6. Error Handling

### 6.1 Error Types (errors.rs)
```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),
    
    #[error("SSH error: {0}")]
    Ssh(String),
    
    #[error("SFTP error: {0}")]
    Sftp(String),
    
    #[error("Encryption error: {0}")]
    Encryption(String),
    
    #[error("Vault error: {0}")]
    Vault(String),
    
    #[error("Validation error: {0}")]
    Validation(String),
    
    #[error("Not found: {0}")]
    NotFound(String),
    
    #[error("Permission denied")]
    PermissionDenied,
    
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),
}

impl serde::Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let error_msg = self.to_string();
        serializer.serialize_str(&error_msg)
    }
}
```

---

## 7. State Management

### 7.1 App State (state.rs)
```rust
use std::sync::Mutex;
use crate::db::Database;
use crate::ssh::SessionManager;
use crate::crypto::Vault;

pub struct AppState {
    pub db: Mutex<Database>,
    pub sessions: Mutex<SessionManager>,
    pub vault: Mutex<Vault>,
}

impl AppState {
    pub fn new(db_path: &str) -> Result<Self, AppError> {
        let db = Database::new(db_path)?;
        let sessions = SessionManager::new();
        let vault = Vault::new();
        
        Ok(Self {
            db: Mutex::new(db),
            sessions: Mutex::new(sessions),
            vault: Mutex::new(vault),
        })
    }
}
```

---

## 8. Build Configuration

### 8.1 Cargo.toml
```toml
[package]
name = "shellmate"
version = "0.1.0"
edition = "2021"

[dependencies]
tauri = { version = "2", features = [] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
rusqlite = { version = "0.32", features = ["bundled"] }
russh = "0.45"
russh-keys = "0.45"
argon2 = "0.5"
aes-gcm = "0.10"
uuid = { version = "1", features = ["v4"] }
rand = "0.8"
thiserror = "1"
tokio = { version = "1", features = ["full"] }

[build-dependencies]
tauri-build = { version = "2", features = [] }
```

---

## 9. SSH Connection Strategy

### 9.1 Decision: One Connection Per Tab (MVP)

For MVP, **each terminal tab opens its own SSH TCP connection** to the target server. Multiple tabs to the same host = multiple separate SSH connections.

### 9.2 Rationale

| Approach | Pros | Cons |
|----------|------|------|
| **1 connection per tab** (MVP choice) | Simple isolation, independent lifecycle, one tab failure ≠ other tabs affected, easy to reason about resources | Slightly higher resource usage for same-host scenario, more handshakes if user opens many tabs to same server |
| **Shared connection, multiple channels** | Lower overhead per tab to same host (1 TCP conn, N channels via SSH multiplex), faster subsequent tab opens | Complex lifecycle management, channel close ≠ connection close, one connection death kills all tabs to that host, harder to debug |

### 9.3 Constraints That Influenced Choice

- **MVP timeline pressure**: Multi-channel multiplex adds significant complexity (channel state tracking, graceful degradation when host doesn't allow multiplex, etc.)
- **Resource cost is acceptable**: Modern systems handle 10-20 SSH connections fine
- **User mental model**: Closing one tab should not affect other tabs — even tabs to the same host
- **Failure isolation**: Network blip on one tab shouldn't drop all tabs to that host
- **Future flexibility**: Easier to add multiplex later than remove it

### 9.4 Implementation Sketch

```rust
// One SshSession per tab — owns its own russh client + channel + stream
pub struct SshSession {
    pub id: String,           // matches frontend tab_id
    pub host_id: String,
    handle: russh::client::Handle<ClientHandler>,
    channel: russh::Channel<russh::client::Msg>,
    keepalive_task: tokio::task::JoinHandle<()>,
}

// SessionManager holds all active sessions
pub struct SessionManager {
    sessions: RwLock<HashMap<String, Arc<Mutex<SshSession>>>>,
}

impl SessionManager {
    pub async fn open(&self, host_id: &str) -> Result<String> {
        // Always create a fresh connection — no pooling for MVP
        let session = SshSession::connect(host_id).await?;
        let session_id = session.id.clone();
        self.sessions.write().await.insert(session_id.clone(), Arc::new(Mutex::new(session)));
        Ok(session_id)
    }

    pub async fn close(&self, session_id: &str) -> Result<()> {
        if let Some(session) = self.sessions.write().await.remove(session_id) {
            let mut session = session.lock().await;
            session.disconnect().await?;
        }
        Ok(())
    }
}
```

### 9.5 Resource Limits

To prevent runaway resource usage:
- Soft limit: warn user when active sessions exceed 20
- Hard limit: configurable (default 50) — refuse new connection with clear error message
- Per-session memory budget: ~2-5 MB for buffers and state

### 9.6 SFTP Strategy

SFTP runs as a **separate SSH connection** from the parent terminal session. This was chosen over multiplexing on the same connection for simplicity and isolation:

- Opening SFTP from a terminal tab creates a fresh SSH connection with SFTP subsystem
- SFTP and shell connections are independent — closing SFTP doesn't close the shell
- If user opens SFTP without an active terminal, a new dedicated SSH connection is created
- Multiple SFTP windows per session are supported by opening multiple SFTP connections

**Rationale:** Multiplexing SFTP on the same SSH connection as the shell was considered but rejected — it adds lifecycle complexity (channel close ≠ connection close) and a shell disconnect would kill the SFTP browser. The resource cost of an extra SSH connection per SFTP window is acceptable.

### 9.7 Post-1.0 Evaluation

Conditions to revisit and add connection multiplexing:
- User feedback: "I open 5 tabs to same server, want it faster"
- Performance profiling shows handshake overhead is significant
- Memory pressure on systems with many tabs

If revisited, implement as **opt-in setting** ("Reuse SSH connection for tabs to same host"), not default — to preserve isolation guarantees for users who depend on them.

---

## 10. New Backend Modules (v1.0)

The following modules are added in later phases. Listed here for backend architecture completeness.

### 10.1 `src-tauri/src/mosh/` (Phase 6)

UDP-based State Synchronization Protocol fallback for unreliable networks.

```
mosh/
├── mod.rs           # Module entry, ConnectionFactory
├── handshake.rs     # Spawn mosh-server via SSH, exchange session key
├── transport.rs     # UDP send/receive, sequence numbers, retransmits
├── state_sync.rs    # SSP terminal state sync (predictive echo)
└── session.rs       # Mosh session lifecycle (mirrors ssh::session API)
```

**Integration:** `MoshSession` implements same `SessionTrait` as `SshSession` so frontend doesn't care about transport. Tab metadata indicates protocol.

**Mosh-server bootstrap:** initial connection over SSH to spawn mosh-server, parse `MOSH CONNECT <port> <key>` line, then switch to UDP transport.

### 10.2 `src-tauri/src/sync/` (Phase 9)

Multi-device sync engine with pluggable backends.

```
sync/
├── mod.rs           # SyncEngine, public API
├── manifest.rs      # Encrypted manifest format, version vectors
├── encrypt.rs       # Per-payload encryption (AES-GCM with sync key)
├── conflict.rs      # Conflict detection + resolution
├── change_log.rs    # Local change tracker (entity → version)
└── backend/
    ├── mod.rs       # SyncBackend trait
    ├── icloud.rs    # iCloud KVS / CloudKit
    ├── gdrive.rs    # Google Drive API v3
    ├── dropbox.rs   # Dropbox API v2
    ├── s3.rs        # S3-compatible (AWS / MinIO / Backblaze B2)
    ├── webdav.rs    # WebDAV (Nextcloud / generic)
    └── http.rs      # Self-hosted HTTP with bearer token
```

**Tauri commands:**
- `sync_status` — last sync time, queued changes, error state
- `sync_configure` — set backend + credentials
- `sync_now` — manual trigger
- `sync_pause` / `sync_resume`
- `sync_resolve_conflict` — accept local / remote / merged

### 10.3 `src-tauri/src/plugin/` (Phase 12)

Wasmtime-based plugin runtime.

```
plugin/
├── mod.rs           # PluginManager, public API
├── manifest.rs      # Manifest parsing + signature verification
├── runtime.rs       # Wasmtime Store, Engine, Linker setup
├── host_api.rs      # Host functions exposed to plugins
├── permissions.rs   # Capability gating
└── registry.rs      # Installed plugin registry (local SQLite table)
```

**Tauri commands:**
- `plugin_list` — installed plugins + state
- `plugin_install` — load `.wasm` + manifest, verify signature, prompt user
- `plugin_uninstall`
- `plugin_enable` / `plugin_disable`
- `plugin_grant_capability` / `plugin_revoke_capability`

**Host API surface (capability-gated):**
- `log(level, message)` — always allowed
- `register_panel(panel_def)` — needs `panel`
- `terminal_data_in(session_id, callback)` — needs `terminal_data`
- `terminal_data_out(session_id, callback)` — needs `terminal_data`
- `pre_connect(callback)` / `post_connect(callback)` — hooks
- `net_request(url)` — needs `network` (allow-listed)
- `fs_read(path)` / `fs_write(path)` — needs `filesystem` (scoped)
- `secret_get(host_id)` — needs `secrets` + per-access user prompt

### 10.4 `src-tauri/src/audit/` (Phase 13)

Encrypted hash-chained audit log.

```
audit/
├── mod.rs           # AuditLog, public API
├── event.rs         # Event types + serialization
├── chain.rs         # Hash chain (prev_hash field)
├── redaction.rs     # Pattern-based secret redaction
└── export.rs        # Signed JSONL export
```

**Tauri commands:**
- `audit_query` — filter/search events
- `audit_export` — write signed JSONL to file
- `audit_purge` — apply retention policy (delete events older than threshold)
- `audit_set_host_enabled` — opt-in per host

### 10.5 `src-tauri/src/sftp/` (Phase 5)

Already planned in original spec; full implementation in Phase 5.

### 10.6 `src-tauri/src/team/` (Phase 11)

Team vault: shared host configs encrypted with team key.

```
team/
├── mod.rs           # TeamManager, public API
├── keypair.rs       # X25519 keypair management (per-user)
├── team_key.rs      # Team master key generation, wrap/unwrap per member
├── share.rs         # Per-host share envelope
└── rotation.rs      # Team key rotation on member revoke
```

**Tauri commands:**
- `team_create` — initialize team, generate master key
- `team_invite` — generate invite payload (encrypts team master key with invitee's pubkey)
- `team_accept_invite` — decrypt + store team master key
- `team_revoke_member` — rotate team key, re-encrypt all shared hosts
- `team_share_host` — wrap host config with team key
- `team_list_shared`

### 10.7 `src-tauri/src/biometric/` (Phase 8)

Per-OS biometric wrapping of vault key. Implementation routed via `cfg`.

```
biometric/
├── mod.rs           # BiometricVault trait
├── macos.rs         # LocalAuthentication framework
├── ios.rs           # LocalAuthentication framework
├── windows.rs       # Windows Hello via WinRT
├── android.rs       # BiometricPrompt JNI
└── stub.rs          # Linux fallback (returns Unsupported)
```

### 10.8 Backend Module Map (Complete)

```
src-tauri/src/
├── main.rs
├── lib.rs
├── state.rs
├── errors.rs
│
├── commands/        # Tauri command handlers (frontend boundary)
├── crypto/          # Argon2id, AES-GCM, SecureBuffer
├── db/              # SQLite + SQLCipher (Phase 7)
├── vault/           # Master password + vault state
├── ssh/             # russh sessions (1 conn / tab)
├── mosh/            # Mosh fallback (Phase 6)
├── sftp/            # SFTP client (Phase 5)
├── port_forward/    # SSH tunnel rules (Phase 5)
├── sync/            # Multi-device sync (Phase 9)
├── plugin/          # Wasmtime sandbox (Phase 12)
├── audit/           # Audit log (Phase 13)
├── team/            # Team vault (Phase 11)
├── biometric/       # Per-OS biometric (Phase 8)
├── theme/           # Custom theme storage (Phase 4)
└── platform/        # Platform-specific glue (mobile / desktop)
```

---

*This document outlines the complete backend architecture and implementation plan for ShellMate.*
