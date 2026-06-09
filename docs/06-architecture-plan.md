# Architecture Plan
## ShellMate - System Architecture

**Version:** 1.1
**Last Updated:** 2026-06-09

---

## 1. System Architecture Overview

### 1.1 High-Level Architecture
```
┌─────────────────────────────────────────────────────────────────┐
│                        ShellMate App                            │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │                    Frontend (React)                      │   │
│  │  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐   │   │
│  │  │ Terminal  │ │   Host   │ │   SFTP   │ │ Settings │   │   │
│  │  │ Manager  │ │ Manager  │ │ Browser  │ │  Dialog  │   │   │
│  │  └────┬─────┘ └────┬─────┘ └────┬─────┘ └────┬─────┘   │   │
│  │       │            │            │            │           │   │
│  │       └────────────┼────────────┼────────────┘           │   │
│  │                    │            │                        │   │
│  │              ┌─────▼────────────▼─────┐                 │   │
│  │              │     Tauri Bridge        │                 │   │
│  │              │   (invoke / events)     │                 │   │
│  │              └────────────┬────────────┘                 │   │
│  └───────────────────────────┼───────────────────────────────┘   │
│                              │                                   │
│  ┌───────────────────────────▼───────────────────────────────┐   │
│  │                    Backend (Rust)                          │   │
│  │  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐   │   │
│  │  │   SSH    │ │   SFTP   │ │  Vault   │ │ Database │   │   │
│  │  │ Handler  │ │  Client  │ │ Handler  │ │  Layer   │   │   │
│  │  └────┬─────┘ └────┬─────┘ └────┬─────┘ └────┬─────┘   │   │
│  │       │            │            │            │           │   │
│  │       └────────────┼────────────┼────────────┘           │   │
│  │                    │            │                        │   │
│  │              ┌─────▼────────────▼─────┐                 │   │
│  │              │      SQLite Database    │                 │   │
│  │              │    (Encrypted Storage)  │                 │   │
│  │              └────────────────────────┘                 │   │
│  └───────────────────────────────────────────────────────────┘   │
│                              │                                   │
└──────────────────────────────┼───────────────────────────────────┘
                               │
                    ┌──────────▼──────────┐
                    │   Remote SSH Server  │
                    │   (TCP/IP Network)   │
                    └─────────────────────┘
```

---

## 2. Component Architecture

### 2.1 Frontend Components

#### Layout Layer
```
AppLayout
├── TitleBar (custom window controls)
├── Sidebar
│   ├── SearchBar
│   ├── HostList
│   │   ├── HostGroup (expandable)
│   │   │   └── HostItem
│   │   └── HostItem (ungrouped)
│   ├── SnippetLink
│   └── SettingsLink
├── MainContent
│   ├── TabBar
│   │   └── Tab (per terminal)
│   └── ContentArea
│       ├── TerminalManager
│       │   └── Terminal (xterm.js)
│       ├── SftpBrowser
│       └── SettingsDialog
└── StatusBar
```

#### Terminal Layer
```
TerminalManager
├── ActiveTerminal
│   └── Terminal
│       ├── xterm.js Instance
│       ├── FitAddon (resize)
│       ├── SearchAddon (search)
│       └── WebLinksAddon (links)
├── TerminalTabs
└── TerminalControls
```

### 2.2 Backend Components

#### SSH Layer
```
SshManager
├── ConnectionPool
│   └── SshConnection[]
├── SessionManager
│   └── SshSession[]
├── AuthManager
│   ├── PasswordAuth
│   └── KeyAuth
└── KeepAliveManager
```

#### Storage Layer
```
StorageManager
├── Database
│   ├── HostRepository
│   ├── GroupRepository
│   ├── CredentialRepository
│   ├── SnippetRepository
│   └── SettingsRepository
├── Vault
│   ├── KeyDerivation (Argon2id)
│   ├── Encryption (AES-256-GCM)
│   └── MemoryProtection (zeroize)
└── Cache
    ├── HostCache
    └── CredentialCache
```

---

## 3. Data Flow Architecture

### 3.1 SSH Connection Flow
```
┌──────────────┐     ┌──────────────┐     ┌──────────────┐
│   Frontend   │     │    Tauri     │     │    Rust      │
│   (React)    │     │   Bridge     │     │  Backend     │
└──────┬───────┘     └──────┬───────┘     └──────┬───────┘
       │                    │                    │
       │ 1. User clicks     │                    │
       │    "Connect"       │                    │
       │───────────────────>│                    │
       │                    │ 2. invoke()        │
       │                    │    ssh_connect     │
       │                    │───────────────────>│
       │                    │                    │
       │                    │                    │ 3. Get host from DB
       │                    │                    │ 4. Get credential from vault
       │                    │                    │ 5. Decrypt credential
       │                    │                    │ 6. Establish SSH connection
       │                    │                    │ 7. Create session
       │                    │                    │
       │                    │ 8. Return session  │
       │                    │    ID              │
       │                    │<───────────────────│
       │ 9. Session ID      │                    │
       │<───────────────────│                    │
       │                    │                    │
       │ 10. Initialize     │                    │
       │     terminal       │                    │
       │                    │                    │
       │ 11. User types     │                    │
       │     command        │                    │
       │───────────────────>│ 12. ssh_send       │
       │                    │───────────────────>│
       │                    │                    │ 13. Send to SSH server
       │                    │                    │
       │                    │ 14. ssh_output     │
       │                    │    event           │
       │                    │<───────────────────│
       │ 15. Render         │                    │
       │     output         │                    │
       │<───────────────────│                    │
```

### 3.2 Credential Retrieval Flow
```
┌──────────────┐     ┌──────────────┐     ┌──────────────┐
│   Frontend   │     │    Vault     │     │  Database    │
└──────┬───────┘     └──────┬───────┘     └──────┬───────┘
       │                    │                    │
       │ 1. get_credential  │                    │
       │    (id)            │                    │
       │───────────────────>│                    │
       │                    │ 2. Get encrypted   │
       │                    │    data from DB    │
       │                    │───────────────────>│
       │                    │ 3. Return          │
       │                    │    encrypted_data  │
       │                    │    + nonce         │
       │                    │<───────────────────│
       │                    │                    │
       │                    │ 4. Decrypt with    │
       │                    │    vault key       │
       │                    │                    │
       │ 5. Return          │                    │
       │    plaintext       │                    │
       │<───────────────────│                    │
```

---

## 4. Security Architecture

### 4.1 Encryption Layers
```
┌─────────────────────────────────────────────────────────┐
│                    Security Layers                       │
├─────────────────────────────────────────────────────────┤
│                                                          │
│  Layer 1: Master Password                                │
│  ┌─────────────────────────────────────────────────┐   │
│  │  User Input → Argon2id → Derived Key (256-bit) │   │
│  └─────────────────────────────────────────────────┘   │
│                                                          │
│  Layer 2: Vault Encryption                               │
│  ┌─────────────────────────────────────────────────┐   │
│  │  Credentials → AES-256-GCM → Encrypted BLOB    │   │
│  └─────────────────────────────────────────────────┘   │
│                                                          │
│  Layer 3: Memory Protection                              │
│  ┌─────────────────────────────────────────────────┐   │
│  │  Plaintext Credentials → zeroize on drop        │   │
│  └─────────────────────────────────────────────────┘   │
│                                                          │
│  Layer 4: Transport Security                             │
│  ┌─────────────────────────────────────────────────┐   │
│  │  SSH Connection → Encrypted TCP (SSH Protocol)  │   │
│  └─────────────────────────────────────────────────┘   │
│                                                          │
└─────────────────────────────────────────────────────────┘
```

### 4.2 Trust Boundaries
```
┌─────────────────────────────────────────────────────────┐
│                    Trust Boundaries                       │
├─────────────────────────────────────────────────────────┤
│                                                          │
│  ┌─────────────────────────────────────────────────┐   │
│  │  Trusted Zone (App Process)                      │   │
│  │  - Decrypted credentials (memory only)           │   │
│  │  - Encryption keys                               │   │
│  │  - Session state                                 │   │
│  └─────────────────────────────────────────────────┘   │
│                                                          │
│  ┌─────────────────────────────────────────────────┐   │
│  │  Untrusted Zone (Storage)                        │   │
│  │  - SQLite database (encrypted credentials)       │   │
│  │  - Config files                                  │   │
│  │  - Log files (no credentials)                    │   │
│  └─────────────────────────────────────────────────┘   │
│                                                          │
│  ┌─────────────────────────────────────────────────┐   │
│  │  External Zone (Network)                         │   │
│  │  - SSH connections (encrypted)                   │   │
│  │  - No telemetry/analytics                        │   │
│  └─────────────────────────────────────────────────┘   │
│                                                          │
└─────────────────────────────────────────────────────────┘
```

---

## 5. Error Handling Architecture

### 5.1 Error Categories
```rust
pub enum AppError {
    // Recoverable errors (show toast, retry)
    Network(NetworkError),
    Ssh(SshError),
    Sftp(SftpError),
    
    // Critical errors (show dialog, require action)
    Database(DatabaseError),
    Vault(VaultError),
    Crypto(CryptoError),
    
    // User errors (show validation message)
    Validation(ValidationError),
    NotFound(NotFoundError),
}
```

### 5.2 Error Propagation
```
┌──────────────┐     ┌──────────────┐     ┌──────────────┐
│   Backend    │     │    Tauri     │     │   Frontend   │
│   (Rust)     │     │   Bridge     │     │   (React)    │
└──────┬───────┘     └──────┬───────┘     └──────┬───────┘
       │                    │                    │
       │ 1. Error occurs    │                    │
       │───────────────────>│                    │
       │                    │ 2. Serialize error │
       │                    │    to JSON         │
       │                    │───────────────────>│
       │                    │                    │ 3. Parse error
       │                    │                    │ 4. Show toast/dialog
       │                    │                    │ 5. Update UI state
```

---

## 6. State Management Architecture

### 6.1 Global State Flow
```
┌─────────────────────────────────────────────────────────┐
│                    State Management                       │
├─────────────────────────────────────────────────────────┤
│                                                          │
│  ┌─────────────────────────────────────────────────┐   │
│  │  Zustand Stores (Frontend)                       │   │
│  │  ┌──────────┐ ┌──────────┐ ┌──────────┐        │   │
│  │  │ Host     │ │ Tab      │ │ Vault    │        │   │
│  │  │ Store    │ │ Store    │ │ Store    │        │   │
│  │  └────┬─────┘ └────┬─────┘ └────┬─────┘        │   │
│  │       │            │            │               │   │
│  │       └────────────┼────────────┘               │   │
│  │                    │                            │   │
│  │              ┌─────▼────────────┐               │   │
│  │              │   Tauri invoke   │               │   │
│  │              └──────────────────┘               │   │
│  └─────────────────────────────────────────────────┘   │
│                                                          │
│  ┌─────────────────────────────────────────────────┐   │
│  │  App State (Backend)                             │   │
│  │  ┌──────────┐ ┌──────────┐ ┌──────────┐        │   │
│  │  │ Database │ │ Sessions │ │ Vault    │        │   │
│  │  │ (Mutex)  │ │ (Mutex)  │ │ (Mutex)  │        │   │
│  │  └──────────┘ └──────────┘ └──────────┘        │   │
│  └─────────────────────────────────────────────────┘   │
│                                                          │
└─────────────────────────────────────────────────────────┘
```

---

## 7. Performance Architecture

### 7.1 Caching Strategy
```
┌─────────────────────────────────────────────────────────┐
│                    Caching Layers                         │
├─────────────────────────────────────────────────────────┤
│                                                          │
│  L1: In-Memory Cache (Frontend)                          │
│  - Host list (frequently accessed)                       │
│  - Snippet list                                          │
│  - Settings                                              │
│                                                          │
│  L2: Session Cache (Backend)                             │
│  - Active SSH sessions                                   │
│  - Decrypted credentials (memory only)                   │
│  - Terminal state                                        │
│                                                          │
│  L3: Database (Persistent)                               │
│  - All data stored encrypted                             │
│  - SQLite WAL mode for concurrent reads                  │
│                                                          │
└─────────────────────────────────────────────────────────┘
```

### 7.2 Resource Management
```
┌─────────────────────────────────────────────────────────┐
│                    Resource Limits                        │
├─────────────────────────────────────────────────────────┤
│                                                          │
│  Memory:                                                 │
│  - Idle: < 50MB                                          │
│  - 5 tabs: < 100MB                                       │
│  - Credentials: zeroized after use                       │
│                                                          │
│  CPU:                                                    │
│  - SSH I/O: async (tokio)                                │
│  - Terminal: batched updates                             │
│  - UI: virtualized lists                                 │
│                                                          │
│  Network:                                                │
│  - SSH keepalive: 60s                                    │
│  - Connection pooling: per-host                          │
│                                                          │
│  Disk:                                                   │
│  - Database: single file                                 │
│  - Logs: rotated, max 10MB                               │
│                                                          │
└─────────────────────────────────────────────────────────┘
```

---

## 8. Cross-Platform Architecture

### 8.1 Platform Abstraction
```
┌─────────────────────────────────────────────────────────┐
│                    Platform Support                       │
├─────────────────────────────────────────────────────────┤
│                                                          │
│  ┌─────────────────────────────────────────────────┐   │
│  │  Common Layer (Cross-platform)                   │   │
│  │  - React UI                                      │   │
│  │  - Rust backend logic                            │   │
│  │  - SQLite database                               │   │
│  │  - SSH/SFTP implementation                       │   │
│  └─────────────────────────────────────────────────┘   │
│                                                          │
│  ┌─────────────────────────────────────────────────┐   │
│  │  Platform Layer (OS-specific)                    │   │
│  │  ┌──────────┐ ┌──────────┐ ┌──────────┐        │   │
│  │  │ Windows  │ │  macOS   │ │  Linux   │        │   │
│  │  │ MSI      │ │  DMG     │ │ AppImage │        │   │
│  │  │ .exe     │ │  .app    │ │  .deb    │        │   │
│  │  └──────────┘ └──────────┘ └──────────┘        │   │
│  └─────────────────────────────────────────────────┘   │
│                                                          │
└─────────────────────────────────────────────────────────┘
```

### 8.2 File System Paths
```rust
pub fn get_app_data_dir() -> PathBuf {
    match std::env::consts::OS {
        "windows" => {
            let appdata = std::env::var("APPDATA").unwrap();
            PathBuf::from(appdata).join("ShellMate")
        }
        "macos" => {
            let home = std::env::var("HOME").unwrap();
            PathBuf::from(home)
                .join("Library")
                .join("Application Support")
                .join("ShellMate")
        }
        "linux" => {
            let home = std::env::var("HOME").unwrap();
            PathBuf::from(home).join(".config").join("shellmate")
        }
        _ => panic!("Unsupported OS"),
    }
}
```

---

## 9. Testing Architecture

### 9.1 Test Pyramid
```
┌─────────────────────────────────────────────────────────┐
│                    Testing Strategy                       │
├─────────────────────────────────────────────────────────┤
│                                                          │
│  ┌─────────────────────────────────────────────────┐   │
│  │  E2E Tests (10%)                                 │   │
│  │  - Full SSH connection flow                      │   │
│  │  - Multi-tab operations                          │   │
│  │  - SFTP file operations                          │   │
│  └─────────────────────────────────────────────────┘   │
│                                                          │
│  ┌─────────────────────────────────────────────────┐   │
│  │  Integration Tests (30%)                         │   │
│  │  - Terminal ↔ SSH integration                    │   │
│  │  - Vault ↔ Database integration                  │   │
│  │  - Frontend ↔ Backend communication              │   │
│  └─────────────────────────────────────────────────┘   │
│                                                          │
│  ┌─────────────────────────────────────────────────┐   │
│  │  Unit Tests (60%)                                │   │
│  │  - SSH connection logic                          │   │
│  │  - Encryption/decryption                         │   │
│  │  - Database queries                              │   │
│  │  - UI components                                 │   │
│  │  - State management                              │   │
│  └─────────────────────────────────────────────────┘   │
│                                                          │
└─────────────────────────────────────────────────────────┘
```

---

## 10. Deployment Architecture

### 10.1 Build Pipeline
```
┌──────────────┐     ┌──────────────┐     ┌──────────────┐
│    Source    │     │    Build     │     │   Release    │
│    Code      │     │   Process    │     │   Artifacts  │
└──────┬───────┘     └──────┬───────┘     └──────┬───────┘
       │                    │                    │
       │ 1. Git push        │                    │
       │───────────────────>│                    │
       │                    │ 2. CI/CD           │
       │                    │    - Lint          │
       │                    │    - Test          │
       │                    │    - Build         │
       │                    │───────────────────>│
       │                    │                    │
       │                    │                    │ 3. Artifacts
       │                    │                    │    - Windows MSI
       │                    │                    │    - macOS DMG
       │                    │                    │    - Linux AppImage
```

---

*This document outlines the complete system architecture for ShellMate.*
