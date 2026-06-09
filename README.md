
<picture>
  <source media="(prefers-color-scheme: dark)" srcset="https://img.shields.io/badge/ShellMate-v0.1.0-7c3aed?style=for-the-badge&logo=rust&logoColor=white&labelColor=1e1b4b">
  <img alt="ShellMate" src="https://img.shields.io/badge/ShellMate-v0.1.0-7c3aed?style=for-the-badge&logo=rust&logoColor=white&labelColor=1e1b4b">
</picture>

# ShellMate 🐚

**Multi-SSH client desktop app — self-hosted, local-first, encrypted.**

Connect to multiple SSH servers simultaneously in one window. No subscriptions, no cloud dependency, no telemetry. Your servers, your keys, your machine.

![Tech Stack](https://img.shields.io/badge/Tauri_v2-FFC131?logo=tauri&logoColor=191923) ![React 18](https://img.shields.io/badge/React_18-61DAFB?logo=react&logoColor=191923) ![TypeScript](https://img.shields.io/badge/TypeScript_Strict-3178C6?logo=typescript&logoColor=white) ![Rust](https://img.shields.io/badge/Rust-000000?logo=rust&logoColor=white) ![SQLite](https://img.shields.io/badge/SQLite-003B57?logo=sqlite&logoColor=white)

---

## Why ShellMate?

SSH clients like Termius lock essential features (multiple hosts, snippets, sync) behind expensive subscriptions. Free alternatives like PuTTY haven't aged well.

ShellMate is built for developers and sysadmins who need:

- **Multi-SSH connection** — Many servers, one window, independent tabs.
- **Local-first** — Data stays on your machine. No third-party servers.
- **Encrypted vault** — AES-256-GCM + Argon2id for credential storage.
- **Cross-platform** — Windows, macOS, Linux. Mobile (Android/iOS) coming post-MVP.
- **Modern UX** — Dark theme, keyboard-first, minimal UI.

## Features

### Current (Phase 1 — Project Setup ✅)

| Feature | Status |
|---------|--------|
| Tauri v2 + React 18 + Vite 6 scaffold | ✅ |
| Custom title bar & window layout | ✅ |
| Sidebar with host list & groups | ✅ |
| Multi-tab session bar with status indicators | ✅ |
| Status bar (vault state, connections) | ✅ |
| Zustand stores (host, tab, UI) | ✅ |
| SQLite schema + migration runner | ✅ |
| Host CRUD (create, read, update, delete) | ✅ |
| Settings store & commands | ✅ |
| ESLint + Prettier + strict TypeScript | ✅ |
| xterm.js dependencies | ✅ |

### In Progress (Phase 2–4)

- Vault: Argon2id + AES-256-GCM encryption
- SSH connection via `russh` (password & key auth)
- xterm.js terminal with I/O streaming
- Multi-tab SSH sessions
- Snippets / command shortcuts
- SFTP file browser
- Port forwarding

### Planned (Post-MVP)

- Mobile app (Android & iOS via Tauri v2 mobile target)
- Multi-device sync (iCloud, GDrive, S3, WebDAV)
- Biometric unlock (Touch ID, Windows Hello)
- Broadcast mode — send commands to many servers at once

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
| App Framework | [Tauri v2](https://v2.tauri.app/) — ~5MB binary, native WebView |
| Frontend | React 18 + Vite 6 + TypeScript (strict) |
| Styling | Tailwind CSS 3 + shadcn/ui |
| Terminal | [xterm.js](https://xtermjs.org/) — industry standard |
| SSH Backend | Rust via [`russh`](https://crates.io/crates/russh) crate |
| Local Storage | SQLite via [`rusqlite`](https://crates.io/crates/rusqlite) |
| Encryption | AES-256-GCM + Argon2id |
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
│   ├── components/layout/      # App shell components
│   ├── stores/                 # Zustand state stores
│   ├── lib/                    # Utilities & Tauri invoke wrappers
│   ├── types/                  # TypeScript types
│   └── styles/                 # Global CSS
├── src-tauri/                  # Rust backend
│   ├── src/
│   │   ├── commands/           # Tauri command handlers
│   │   ├── db/                 # SQLite schema & migrations
│   │   ├── ssh/                # SSH connection (planned)
│   │   ├── crypto/             # Encryption (planned)
│   │   └── sftp/               # SFTP (planned)
│   ├── icons/                  # App icons
│   └── Cargo.toml
├── docs/                       # Planning & architecture docs
├── PRD.md                      # Product requirements
└── CHANGELOG.md
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
