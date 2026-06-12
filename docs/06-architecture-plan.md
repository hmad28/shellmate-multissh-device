# Architecture Plan
## ShellMate — System Architecture (v1.0 Production)

**Version:** 2.3
**Last Updated:** 2026-06-11

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
│  - Idle: < 80MB (desktop)                               │
│  - 5 tabs: < 150MB (desktop)                            │
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

---

## 11. Multi-Platform Architecture (Mobile + Desktop)

ShellMate v1.0 ships on Windows, macOS, Linux, Android, and iOS using Tauri v2's mobile target.

### 11.1 Shared Core, Adaptive UI

```
┌─────────────────────────────────────────────────────────┐
│             Shared Rust Core (Library)                  │
│  vault │ crypto │ db │ ssh │ sftp │ sync │ plugin │ ... │
└─────────────────────────┬───────────────────────────────┘
                          │
            ┌─────────────┴─────────────┐
            │                           │
   ┌────────▼────────┐         ┌────────▼────────┐
   │   Desktop Shell │         │   Mobile Shell  │
   │   (Tauri v2)    │         │   (Tauri v2)    │
   └────────┬────────┘         └────────┬────────┘
            │                           │
   ┌────────▼────────┐         ┌────────▼────────┐
   │  React (WebView)│         │  React (WebView)│
   │  Desktop layout │         │  Mobile layout  │
   │  Keyboard-first │         │  Touch-first    │
   └─────────────────┘         └─────────────────┘
```

The Rust library (`shellmate_lib`) is identical across platforms. Only the shell crate (entry point) and React layout differ. UI components detect viewport size and capability flags via a single `usePlatform()` hook.

### 11.2 Mobile-Specific Surface

- **Extended key bar**: persistent row above virtual keyboard (Esc, Tab, Ctrl, Alt, ↑↓←→, |, ~, -, /, configurable)
- **Bottom-sheet navigation**: tab switcher, host list, settings — all use bottom-sheet pattern
- **Full-screen panels**: SFTP, snippet picker, settings open as full-screen modals
- **Gesture handling**: swipe between tabs, pinch-to-zoom for terminal font, long-press for context menu
- **Background lifecycle**: keep SSH sessions alive briefly when app backgrounds; notification on disconnect
- **Biometric integration**: Phase 8 plugin per OS (Tauri biometric plugin or custom)

### 11.3 Platform Capability Detection

```rust
// Backend: cfg flags route to per-platform impls
#[cfg(target_os = "android")]
use crate::platform::android::biometric;
#[cfg(target_os = "ios")]
use crate::platform::ios::biometric;
#[cfg(target_os = "macos")]
use crate::platform::macos::biometric;
```

```typescript
// Frontend: runtime capability via Tauri API
import { platform } from '@tauri-apps/plugin-os';
const isMobile = ['android', 'ios'].includes(platform());
```

---

## 12. Sync Architecture (Multi-Device, E2E Encrypted)

```
┌──────────────────────────────────────────────────────────┐
│                   Device A (Desktop)                     │
│                                                          │
│  ShellMate App                                           │
│  └─ Sync Engine                                          │
│     ├─ Change tracker (vector clocks per entity)         │
│     ├─ Encrypt-then-upload (vault key derivative)        │
│     └─ Backend adapter (iCloud/GDrive/S3/WebDAV/HTTP)    │
└─────────────────────────┬────────────────────────────────┘
                          │ HTTPS (encrypted payloads)
                ┌─────────▼──────────┐
                │   User's own cloud │
                │ (provider can NOT  │
                │  read payloads)    │
                └─────────┬──────────┘
                          │ HTTPS
┌─────────────────────────▼────────────────────────────────┐
│                   Device B (Mobile)                      │
│                                                          │
│  ShellMate App                                           │
│  └─ Sync Engine                                          │
│     └─ Decrypt-then-merge                                │
└──────────────────────────────────────────────────────────┘
```

### 12.1 Encryption Model

- **Sync key**: derived from master password via HKDF (separate from vault key, but tied to same master password)
- **Per-payload nonce**: generated fresh on each upload
- **Object naming**: opaque IDs (UUIDs), no human-readable structure
- **Manifest file**: also encrypted, lists object IDs and version vectors

### 12.2 Conflict Resolution

- **Last-write-wins** for simple cases (e.g., snippet edited on two devices, newer wins)
- **Manual merge UI** when both sides have non-trivial changes (host config, group hierarchy)
- **Version vectors** track origin device + sequence number per entity

### 12.3 Selective Sync

User picks per-device: which hosts/snippets/groups participate. Filter applied at engine level — non-synced entities never leave the device.

### 12.4 Diagnostic Panel

- Last successful sync timestamp
- Pending queue (entities waiting to upload)
- Errors with actionable messages
- Manual "sync now" button
- Force re-encrypt and re-upload (recovery)

---

## 13. Plugin Architecture (Wasmtime Sandbox)

### 13.1 Runtime Model

```
┌──────────────────────────────────────────────────────────┐
│                    ShellMate Host                        │
│                                                          │
│  Plugin Loader                                           │
│  ├─ Verify manifest signature                            │
│  ├─ Check capability grants                              │
│  └─ Instantiate Wasmtime Store                           │
│                                                          │
│         ┌──────────────────────────────────┐             │
│         │   Wasmtime Sandbox Per Plugin    │             │
│         │   ┌────────────────────────┐     │             │
│         │   │   Plugin .wasm module  │     │             │
│         │   └─────────┬──────────────┘     │             │
│         │             │                    │             │
│         │   Host APIs (capability-gated):  │             │
│         │   • terminal_data_in/out filter  │             │
│         │   • pre/post_connect hooks       │             │
│         │   • register_panel               │             │
│         │   • log (always allowed)         │             │
│         │   • net (opt-in)                 │             │
│         │   • fs (opt-in, scoped)          │             │
│         │   • secrets (opt-in, prompt UI)  │             │
│         └──────────────────────────────────┘             │
└──────────────────────────────────────────────────────────┘
```

### 13.2 Capability Permissions

Plugins declare required capabilities in manifest. User reviews and grants on install. Capabilities can be revoked later.

| Capability | Default | Notes |
|-----------|---------|-------|
| `log` | ✅ Always allowed | No PII / no host data |
| `terminal_data` | Per-install consent | Filter / transform stream |
| `panel` | Per-install consent | Custom UI panel |
| `network` | Opt-in | Outbound HTTP only, allow-listed hosts |
| `filesystem` | Opt-in, scoped | Restricted to `~/Documents/Plugins/<id>/` |
| `secrets` | Opt-in, prompt per access | Vault read-only with explicit user prompt |

### 13.3 Plugin Manifest

```toml
[plugin]
id = "com.example.theme-installer"
name = "Theme Installer"
version = "1.0.0"
author = "Example"
api_version = "1"

[capabilities]
panel = true
filesystem = ["read"]

[signature]
algorithm = "ed25519"
public_key = "..."
signature = "..."
```

---

## 14. Audit Log Architecture

### 14.1 Event Capture

Audit events captured at transition points (session start/end, file transfer commit, command sent if opt-in). Events buffered in memory then flushed to encrypted log file.

```
┌─────────────────────────┐
│  SSH session start      │──┐
├─────────────────────────┤  │
│  SSH session end        │  │
├─────────────────────────┤  │
│  SFTP transfer complete │  ├──> Audit Buffer
├─────────────────────────┤  │     │
│  Command sent (opt-in)  │  │     │ flush every 5s
├─────────────────────────┤  │     ▼
│  Vault unlock/lock      │──┘   Encrypted log file
└─────────────────────────┘       (append-only, hash-chained)
```

### 14.2 Hash-Chained Storage

Each event row includes hash of previous event row, forming an integrity chain. Tampering with any past event invalidates all subsequent hashes — detected on export verification.

### 14.3 Privacy Controls

- Opt-in **per host** (not global)
- Redaction patterns (regex) applied to command history before storage
- Retention policy: configurable, default 90 days
- Export signed JSONL for compliance evidence

---

## 15. Theme Architecture

### 15.1 Theme Tokens

Themes define color tokens for both terminal palette and UI:

```typescript
interface ThemeDefinition {
  id: string;
  name: string;
  base: 'dark' | 'light';
  ui: {
    bg: string;          // app background
    bgSidebar: string;
    bgPanel: string;
    bgElevated: string;
    border: string;
    fg: string;
    fgMuted: string;
    accent: string;
    statusConnected: string;
    statusConnecting: string;
    statusDisconnected: string;
  };
  terminal: {
    background: string;
    foreground: string;
    cursor: string;
    cursorAccent: string;
    selection: string;
    ansi: [string; 16];  // 0-15 ANSI colors
  };
  fontFamily: string;
}
```

### 15.2 Theme Loading

- Built-in themes (dark default, light, high-contrast) shipped in app
- Custom themes stored in SQLite (`themes` table) and synced
- Theme editor: live preview, export to JSON, import from file
- Plugin can ship themes via plugin filesystem capability

---

## 16. Broadcast Mode Architecture

### 16.1 Frontend Aggregator

Frontend tracks set of "broadcast-targeted" tab IDs. Single keyboard input dispatched to all targets:

```
User keystroke
     │
     ▼
┌─────────────────────────┐
│ Broadcast Aggregator    │
│ targets: [tab1, tab3]   │
└─────┬─────────┬─────────┘
      │         │
      ▼         ▼
   ssh_send   ssh_send
   (tab1)    (tab3)
```

### 16.2 Visual Indicators

- Broadcast target tabs show distinct border/icon
- Single broadcast input bar with target chips
- Easy add/remove targets via click

### 16.3 Safety

- Broadcast mode disabled by default
- Explicit toggle, persists per-window
- Confirmation prompt for destructive commands (configurable list of dangerous patterns: `rm -rf`, `dd`, `mkfs`, etc.)

---
