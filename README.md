
<picture>
  <source media="(prefers-color-scheme: dark)" srcset="https://img.shields.io/badge/ShellMate-v0.1.0-7c3aed?style=for-the-badge&logo=rust&logoColor=white&labelColor=1e1b4b">
  <img alt="ShellMate" src="https://img.shields.io/badge/ShellMate-v0.1.0-7c3aed?style=for-the-badge&logo=rust&logoColor=white&labelColor=1e1b4b">
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

### Roadmap (Phase 4 → 14)

ShellMate is delivered scope-driven (no fixed timeline). Each phase ships when acceptance criteria are met.

| Phase | Area | Highlights |
|-------|------|-----------|
| 4 | Productivity & Settings | Snippets, settings dialog, custom themes, configurable shortcuts, master password change |
| 5 | File Transfer & Network | SFTP browser, drag-and-drop upload, port forwarding (local & remote) |
| 6 | Network Hardening | Known hosts UI, auto-reconnect, **Mosh support**, **broadcast mode** |
| 7 | Full-DB Encryption | **SQLCipher** migration, defense in depth on top of per-credential AES-GCM |
| 8 | Biometric Unlock | Touch ID, Face ID, Windows Hello, Android Fingerprint |
| 9 | Multi-Device Sync (E2E) | iCloud, GDrive, Dropbox, S3, WebDAV adapters, conflict merge UI |
| 10 | Mobile Apps | Android first, iOS next. Extended key bar, bottom-sheet nav, touch-friendly SFTP |
| 11 | Team Vault | Shared host configs via team key, member management, key rotation |
| 12 | Plugin System | Wasmtime sandbox, capability-based permissions, signed manifests |
| 13 | Audit Log | Opt-in per host, encrypted, exportable signed JSONL |
| 14 | Polish & Distribution | Code signing (Authenticode + macOS notarization), Tauri auto-updater, full a11y pass |

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
| Mosh | Rust (planned, Phase 6) |
| Local Storage | SQLite via [`rusqlite`](https://crates.io/crates/rusqlite) + SQLCipher (Phase 7) |
| Encryption | AES-256-GCM + Argon2id |
| Plugin Runtime | Wasmtime (WASM sandbox, Phase 12) |
| State | [Zustand](https://github.com/pmndrs/zustand) — lightweight React state |

### Security Architecture

Credentials **never leave the Rust layer**. The React frontend only works with opaque host IDs — plaintext passwords and private keys exist only in Rust memory and are zeroized after use.

```
Master Password
      ↓ Argon2id
Derived Key (AES-256)
      ↓ Encrypt
Credentials → Encrypted SQLite (AES-256-GCM)
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
│   │   ├── hosts/              # Host & group management UI components
│   │   ├── layout/             # App shell (TitleBar, Sidebar, StatusBar, TabBar)
│   │   ├── terminal/           # xterm.js terminal view and subscription
│   │   ├── ui/                 # Reusable UI primitives (Button, Modal, Form, Confirm)
│   │   └── vault/              # Vault security gate forms
│   ├── stores/                 # Zustand state stores (host, tab, ui, vault)
│   ├── lib/                    # Utilities & typed Tauri invoke wrappers
│   ├── types/                  # TypeScript interface definitions
│   └── styles/                 # Tailwind global configurations
├── src-tauri/                  # Rust backend (Tauri native wrapper)
│   ├── src/
│   │   ├── commands/           # IPC command routes (host, group, credential, vault, ssh)
│   │   ├── db/                 # SQLite integration, schema definition, and migration runner
│   │   ├── crypto/             # AES-256-GCM encryption and Argon2id KDF primitives
│   │   ├── ssh/                # russh multi-session tasks and PTY managers
│   │   ├── vault/              # Master lock state machines and secure memory buffer
│   │   └── lib.rs
│   ├── Cargo.toml
│   └── tauri.conf.json
├── docs/                       # Specifications and design plans (v2.0 specs)
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
