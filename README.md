
<picture>
  <source media="(prefers-color-scheme: dark)" srcset="https://img.shields.io/badge/ShellMate-v0.2.0--beta-7c3aed?style=for-the-badge&logo=rust&logoColor=white&labelColor=1e1b4b">
  <img alt="ShellMate" src="https://img.shields.io/badge/ShellMate-v0.2.0--beta-7c3aed?style=for-the-badge&logo=rust&logoColor=white&labelColor=1e1b4b">
</picture>

# ShellMate 🐚

**Multi-SSH client — self-hosted, local-first, encrypted, multi-device.**

Connect to multiple SSH servers simultaneously across desktop and mobile. No subscriptions, no cloud dependency, no telemetry. Your servers, your keys, your machine.

![Tech Stack](https://img.shields.io/badge/Tauri_v2-FFC131?logo=tauri&logoColor=191923) ![React 18](https://img.shields.io/badge/React_18-61DAFB?logo=react&logoColor=191923) ![TypeScript](https://img.shields.io/badge/TypeScript_Strict-3178C6?logo=typescript&logoColor=white) ![Rust](https://img.shields.io/badge/Rust-000000?logo=rust&logoColor=white) ![SQLite](https://img.shields.io/badge/SQLite-003B57?logo=sqlite&logoColor=white)

> Designed as a full v1.0 production release — not an MVP. Desktop and mobile, with optional end-to-end-encrypted sync via your own cloud (iCloud, GDrive, Dropbox, S3, WebDAV, or self-hosted).

---

## Why ShellMate?

SSH clients like Termius lock essential features (multiple hosts, snippets, sync, team) behind expensive subscriptions. Free alternatives like PuTTY haven't aged well, and most don't run on mobile.

ShellMate is built for developers, DevOps, and sysadmins who need:

- **Multi-SSH connection** — Many servers, one window, independent tabs. Broadcast command to many at once.
- **Local-first + privacy by default** — Data stays on your machine. No third-party servers. Full-DB encrypted with SQLCipher, credentials with AES-256-GCM.
- **Multi-device** — Windows, macOS, Linux, Android, iOS. Same vault, same hosts.
- **E2E-encrypted sync** — Optional, via your own cloud. ShellMate has no servers in the loop.
- **Extensible** — Plugin system (WASM sandbox), custom themes, team vault.

## Features

### Phase Status (Production-track v1.0)

| Phase | Area | Status |
|-------|------|--------|
| 1 | Project Setup | ✅ |
| 2 | Core SSH + Crypto Vault | ✅ |
| 3 | Host Management & Persistence | ✅ |
| 4 | Productivity & Settings | ✅ |
| 5 | File Transfer & Network (SFTP, Port Forwarding) | ✅ |
| 6 | Network Hardening (TOFU, Auto-reconnect, Broadcast) | ✅ |
| 7 | Full-DB Encryption (SQLCipher) | ✅ |
| 8 | Biometric Unlock (Windows Hello) | ✅ |
| 9 | Multi-Device Sync (E2E, HTTP + S3) | ✅ |
| 10 | Mobile Apps (Android, adaptive UI) | ✅ |
| 11 | Team Vault | ✅ |
| 12 | Plugin System (Wasmtime) | ✅ |
| 13 | Audit Log | ✅ |
| 14 | Polish & Distribution | ✅ |

All 14 phases complete. See [docs/01-development-plan.md](docs/01-development-plan.md) for detailed deliverables.

### Capability Matrix

| Capability | Status | Notes |
|------------|--------|-------|
| Multi-SSH sessions | Implemented | Independent tabs, broadcast mode |
| Credential vault | Implemented | AES-256-GCM + Argon2id, zeroized memory |
| Full-DB encryption | Implemented | SQLCipher with HKDF key split |
| SFTP file transfer | Implemented | Upload/download with progress events |
| Port forwarding | Implemented | Local (-L) and remote (-R) rules |
| TOFU host verification | Implemented | Fingerprint comparison on mismatch |
| Biometric unlock | Implemented | Windows Hello via TPM-backed key |
| Multi-device sync | Implemented | E2E-encrypted, HTTP + S3 backends |
| Team vault | Implemented | Host sharing, member key wrapping |
| Plugin system | Implemented | Wasmtime WASM sandbox, capability-based |
| Audit log | Implemented | Hash-chained, encrypted, per-host opt-in |
| Mobile support | Implemented | Adaptive UI, Android config |
| Snippets & templates | Implemented | Template variables, execute-to-session |
| Theme system | Implemented | 3 built-in + custom theme support |

### Completed Phases

#### Phase 1: Project Setup ✅
- **Tauri v2 + React 18 + Vite 6 scaffold** with custom frameless window design and layouts.
- **SQLite Database**: Local SQLite storage using `rusqlite` with auto-migration runner.
- **App Layout & Shell**: Custom draggable title bar, sidebar with host lists, status bar, and tab session bar.
- **State Management**: Lightweight state handling via Zustand stores (host, tab, UI, settings).
- **Strict Development Standards**: Clean ESLint, Prettier, and strict TypeScript checks.

#### Phase 2: Core SSH & Crypto/Vault ✅
- **Cryptographic Vault**: OWASP-compliant Argon2id KDF key derivation and AES-256-GCM authenticated encryption.
- **Secure Buffer**: Rust-backed `SecureBuffer` wrapping that zeroizes keys and credentials in memory when dropped.
- **Vault Gates & Setup**: Gated app state requiring initial vault setup and master password unlocking, plus mandatory recovery warning + acknowledgement.
- **SSH Session Manager (`russh`)**: Multi-session support, custom event loops, PTY integration (`xterm-256color`), and keepalive checks.
- **Interactive Terminals**: xterm.js wrapper featuring ResizeObserver auto-fit, WebLinks, and PTY resize signals.
- **QuickConnect**: Instant SSH access form for fast connections without storing credentials.

#### Phase 3: Host Management & Persistence ✅
- **Host CRUD UI**: Add/edit modal with validation, password and SSH key authentication.
- **Group CRUD**: Create, edit, delete groups with preset color swatches and custom hex input.
- **Drag-and-Drop**: Move hosts between groups with native HTML5 drag-and-drop.
- **Host Search**: Free-text search across label, hostname, username, group name, tags, and notes.
- **Right-click Context Menu**: Connect, edit, delete actions per host.
- **Connect from Sidebar**: One-click connect uses saved credentials via vault.
- **Empty States**: Friendly UX for no hosts, no groups, no search results.

#### Phase 4: Productivity & Settings ✅
- **Snippets**: CRUD with template variables (`{{username}}`, `{{host}}`, `{{port}}`, `{{label}}`), search, execute-to-active-session.
- **Settings Dialog**: Tabbed UI (General, Terminal, Vault, Theme).
- **Theme System**: 3 built-in themes (ShellMate Dark, Light, High Contrast) via CSS variables. Tailwind reads tokens from `var(--color-*)` so all components retheme instantly.
- **Custom Themes**: storage backend + API ready (full color-picker editor lands in Phase 14 polish).
- **Auto-Lock**: Frontend polls `vault_check_idle` every 15s, throttled activity ping every 60s on user input.
- **Master Password Change**: Atomic re-encryption of all credentials in a single transaction with old/new key zeroize on every error path.

#### Phase 5: File Transfer & Network ✅
- **SFTP Browser**: Directory listing, navigation, upload/download with real-time progress events for transfers, rename, delete, mkdir. Multiple SFTP panels per connection are supported.
- **Port Forwarding**: Local (-L) and remote (-R) rules per host with conflict detection (binding errors handled gracefully). Toggles enable/disable on the fly.

#### Phase 6: Network Hardening ✅
- **Known Hosts (TOFU)**: Trust-on-first-use host verification with mismatch warning dialogs comparing fingerprints.
- **Auto-Reconnect**: Exponential backoff reconnection loop with manual cancel options and status notifications in the tab area.
- **Broadcast Mode**: Keystroke broadcasting to multiple SSH sessions concurrently with visual group indicators.

#### Phase 1–6 Integration & Stabilization ✅
- Fixed critical compilation errors by upgrading `russh-sftp` to `2.1.2`, removing invalid imports, correcting struct references, and resolving memory safety issues where Mutex guards were held across `.await` points.
- Wired host key verification handshakes in the React layout to trigger the TOFU verification dialog.
- Standardized serializable payload structures between Tauri backend commands and frontend state.

#### Phase 7: Full-DB Encryption (SQLCipher) ✅
- **SQLCipher**: Full database encryption at rest via `bundled-sqlcipher`. Defense in depth on top of per-credential AES-GCM.
- **HKDF Key Derivation**: Single Argon2id pass → HKDF split into vault key + DB key (domain-separated).
- **Key Rotation**: `PRAGMA rekey` for in-place SQLCipher key rotation on master password change.
- **Migration**: Automatic plaintext-to-encrypted migration on first vault unlock (backup as `.db.bak`).

#### Phase 8: Biometric Unlock ✅
- **Windows Hello**: TPM-backed key via `KeyCredentialManager`. Biometric/PIN prompt on each unlock.
- **Key Wrapping**: AES-256-GCM wrapping of vault key with HKDF-derived key from device secret.
- **Cross-platform**: `BiometricProvider` trait with platform dispatch. Stub for Linux.

#### Phase 9: Multi-Device Sync (E2E) ✅
- **Sync Engine**: Version vector clocks for multi-device conflict detection.
- **Backends**: HTTP (self-hosted, bearer token) + S3-compatible (AWS Signature V4).
- **Encryption**: Per-payload AES-256-GCM with HKDF-derived key. Opaque UUID object IDs.
- **Conflict Resolution**: Last-write-wins with version vector comparison.

#### Phase 10: Mobile Apps ✅
- **Adaptive UI**: `useIsMobile` hook, `MobileLayout` with bottom navigation, `BottomNav` (4 tabs).
- **MobileKeyBar**: Extended key bar (Esc, Tab, Ctrl, Alt, arrows, symbols) with modifier toggle.
- **Safe Areas**: CSS utilities for iOS notch/home indicator. Touch scroll optimization.
- **Android Config**: JNI + Android Logger dependencies (cfg-gated).

#### Phase 11: Team Vault ✅
- **Team CRUD**: Create/list/delete teams with random team master key.
- **Member Management**: Add members by public key, revoke (timestamp-based).
- **Host Sharing**: Share hosts with teams, permissions (read/edit).
- **Key Wrapping**: Team key wrapped with vault key (AES-GCM). Per-member wrapping via HKDF.

#### Phase 12: Plugin System ✅
- **Wasmtime v29**: WASM runtime with WASI support. Sandboxed execution.
- **Manifest**: JSON manifest with capability declarations + validation.
- **Capabilities**: 6 permissions (log, panel, terminal_data, network, filesystem, secrets) — all opt-in.
- **Crash Isolation**: Plugin traps caught via `spawn_blocking`, never crash host.

#### Phase 13: Audit Log ✅
- **Hash-Chained Events**: SHA256 chain for tamper detection.
- **Encrypted Storage**: AES-256-GCM encrypted payloads with vault key.
- **Per-Host Opt-In**: Default OFF. Command history separate opt-in.
- **Redaction**: Pattern-based secret redaction before storage.
- **Retention**: Configurable per host (default 90 days). Purge API.

#### Phase 14: Polish & Distribution ✅
- **Toast Notifications**: Zustand store with auto-dismiss (success/error/warning/info).
- **Encrypted Export/Import**: SMEX format, Argon2id + AES-256-GCM, base64 transport.
- **Auto-Updater**: Tauri plugin configured (requires signing keys for production).

### All Phases Complete

All 14 phases of the ShellMate v1.0 development plan have been implemented. See [docs/01-development-plan.md](docs/01-development-plan.md) for detailed deliverables per phase.

Remaining for production release: CI/CD setup, code signing certificates, cross-platform testing, and packaging.

For full details see [PRD.md §10 Milestones](PRD.md) and [docs/01-development-plan.md](docs/01-development-plan.md).

## Tech Stack

```
┌─────────────────────────────────────────┐
│           React UI (WebView)            │
│  xterm.js │ Host Manager │ SFTP Browser │
└──────────────────┬──────────────────────┘
                   │ invoke() / events
┌──────────────────▼──────────────────────┐
│            Rust Backend (Tauri)         │
│  SSH Handler │ SQLite │ Crypto Module   │
└──────────────────┬──────────────────────┘
                   │ SSH Protocol
┌──────────────────▼──────────────────────┐
│          Remote SSH Servers             │
└─────────────────────────────────────────┘
```

| Layer | Technology |
|-------|-----------|
| App Framework | [Tauri v2](https://v2.tauri.app/) — desktop + mobile, native WebView |
| Frontend | React 18 + Vite 6 + TypeScript (strict) |
| Styling | Tailwind CSS 3 + shadcn/ui |
| Terminal | [xterm.js](https://xtermjs.org/) — industry standard |
| SSH Backend | Rust via [`russh`](https://crates.io/crates/russh) crate |
| Local Storage | SQLite via [`rusqlite`](https://crates.io/crates/rusqlite) + SQLCipher |
| SFTP | [`russh-sftp`](https://crates.io/crates/russh-sftp) |
| Encryption | AES-256-GCM + Argon2id + HKDF |
| Sync | HTTP + S3 backends with AWS Sig V4 |
| Plugin Runtime | [Wasmtime](https://wasmtime.dev/) (WASM sandbox) |
| Biometric | Windows Hello via `KeyCredentialManager` |
| State | [Zustand](https://github.com/pmndrs/zustand) — lightweight React state |

### Security Architecture

Credentials **never leave the Rust layer**. The React frontend only works with opaque host IDs — plaintext passwords and private keys exist only in Rust memory and are zeroized after use.

```
Master Password
      ↓ Argon2id
Master Key (256-bit)
      ↓ HKDF split
  ┌───┴───┐
Vault Key  DB Key (SQLCipher)
  ↓
AES-256-GCM per-credential
```

## Getting Started

### Prerequisites

- [Rust](https://rustup.rs/) (MSVC toolchain on Windows)
- [Node.js](https://nodejs.org/) >= 18
- npm (ships with Node.js)

### Development

```bash
# Install frontend dependencies
npm install

# Run in development mode (hot reload)
npm run tauri:dev

# Type-check
npm run typecheck

# Lint
npm run lint

# Build for production
npm run tauri:build
```

### Project Structure

```
shellmate/
├── src/                        # React frontend
│   ├── components/
│   │   ├── connect/            # Quick connection forms
│   │   ├── hosts/              # Host & group management UI
│   │   ├── layout/             # App shell (TitleBar, Sidebar, StatusBar, TabBar, BottomNav, MobileLayout)
│   │   ├── port-forward/       # Port forwarding rules panel
│   │   ├── security/           # Host key verification dialog (TOFU)
│   │   ├── settings/           # Tabbed settings dialog (General, Terminal, Vault, Theme)
│   │   ├── sftp/               # SFTP file browser
│   │   ├── snippets/           # Snippet list, form, execute panel
│   │   ├── terminal/           # xterm.js terminal + broadcast mode + mobile key bar
│   │   ├── ui/                 # Reusable primitives (Button, Modal, Form, Confirm, Toast, CommandPalette)
│   │   ├── vault/              # Vault unlock & setup forms
│   │   ├── history/            # Command history panel
│   │   └── updates/            # Auto-updater toast
│   ├── hooks/                  # Custom hooks (useAutoLock, useIsMobile, useKeyboardShortcuts, etc.)
│   ├── stores/                 # Zustand stores (host, tab, ui, vault, settings, snippet, sftp, port-forward, broadcast, toast, command, history, shortcuts)
│   ├── themes/                 # Built-in theme configurations
│   ├── lib/                    # Utilities, snippet expansion, typed Tauri invoke wrappers
│   ├── types/                  # TypeScript interface definitions (host, ssh, sftp, snippet, theme, sync, team, plugin, audit)
│   └── styles/                 # Tailwind global configuration + safe-area utilities
├── src-tauri/                  # Rust backend (Tauri native wrapper)
│   ├── src/
│   │   ├── commands/           # IPC routes (host, group, vault, ssh, snippet, theme, sftp, port_forward, known_hosts, broadcast, sync, team, plugin, audit, biometric, export, git, history)
│   │   ├── db/                 # SQLite + SQLCipher schema + migration runner
│   │   ├── crypto/             # AES-256-GCM + Argon2id + HKDF primitives
│   │   ├── ssh/                # russh multi-session, PTY, reconnect, broadcast
│   │   ├── sftp/               # russh-sftp subsystem manager
│   │   ├── port_forward/       # Local & remote port forwarding
│   │   ├── known_hosts/        # TOFU host key verification
│   │   ├── vault/              # Master lock state machine + secure memory buffer
│   │   ├── biometric/          # Windows Hello / cross-platform biometric provider
│   │   ├── sync/               # Sync engine + backends (HTTP, S3)
│   │   ├── team/               # Team vault (CRUD, members, shares)
│   │   ├── plugin/             # Wasmtime runtime, manifest, permissions
│   │   ├── audit/              # Hash-chained audit log, redaction
│   │   └── lib.rs
│   ├── Cargo.toml
│   └── tauri.conf.json
├── docs/                       # Specifications and design plans (v2.3 specs)
├── PRD.md                      # Master Product Requirements Document
└── CHANGELOG.md                # Project release and change tracker
```
```

## Documentation

| Document | Description |
|----------|-------------|
| [PRD.md](PRD.md) | Full product requirements |
| [docs/](docs/) | Architecture, security, backend, frontend plans |

## License

[MIT](LICENSE) — free to use, modify, and distribute.

---

<p align="center">
  Built with ❤️ for developers who manage too many servers.
</p>
