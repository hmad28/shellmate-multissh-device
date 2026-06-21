# ShellMate

**Self-hosted, local-first, multi-device SSH client.**

ShellMate is a cross-platform SSH workstation for developers, DevOps, and small teams. Manage many servers in one window, share configs across devices through end-to-end-encrypted sync over your own cloud, and keep every credential on your machine — no SaaS, no telemetry, no subscription gates.

> Status: `v0.2.0-beta` — core SSH, vault, SFTP, port forwarding, sync, mobile, and audit workflows are implemented. Biometric unlock, encrypted team sharing, plugin host APIs, code signing, and CI/CD packaging remain before a tagged `v1.0` release.

---

## Key Features

- ✅ **One command** — install, scan QR, done in 30 seconds
- ✅ **Auto tunnel** — Cloudflare tunnel/Tailscale funnel starts automatically, no port forwarding
- ✅ **All-in-one** — terminal + remote desktop + file explorer + code editor in one app
- ✅ **Works on phone** — full workspace from your browser, under 50ms latency
- ✅ **Persistent sessions** — PTY daemon survives server restarts
- ✅ **Pair Device security** — only approved devices can connect, zero signup friction

---

## 📊 ShellMate vs Other Remote Solutions

| Feature | ShellMate | 9Remote | Claude Remote | Termius | Chrome Remote |
| :--- | :---: | :---: | :---: | :---: | :---: |
| Zero Config | ✅ | ✅ | ✅ | ❌ | ✅ |
| Terminal Access | ✅ | ✅ | ✅ | ✅ | ❌ |
| Remote Desktop | ✅ | ✅ | ❌ | ❌ | ✅ |
| File Explorer | ✅ | ✅ | ❌ | ✅ | ❌ |
| Code Editor | ✅ | ✅ | ❌ | ❌ | ❌ |
| Git Integration | ✅ | ✅ | ❌ | ❌ | ❌ |
| Mobile Optimized | ✅ | ✅ | ✅ | ✅ | ❌ |
| Browser-Based | ✅ | ✅ | ✅ | ❌ | ✅ |
| QR Login | ✅ | ✅ | ✅ | ❌ | ❌ |
| Auto Tunnel | ✅ | ✅ | ✅ | ❌ | ✅ |
| Persistent Sessions | ✅ | ✅ | ✅ | ✅ | ❌ |
| Multi-Device Sync | ✅ | ✅ | ✅ | ✅ | ❌ |
| Push Notifications | ✅ | ✅ | ✅ | ❌ | ❌ |
| AI Integration | ✅ | ✅ | ✅ | ❌ | ❌ |
| No Port Forwarding | ✅ | ✅ | ✅ | ❌ | ✅ |
| No Account Required | ✅ | ✅ | ❌ | ❌ | ❌ |
| **TOTAL** | **16 / 16** | **16 / 16** | **11 / 16** | **7 / 16** | **5 / 16** |

---

## Highlights

- **Multi-session terminal** — unlimited SSH tabs, keepalive, auto-reconnect with exponential backoff, broadcast keystrokes across selected sessions.
- **Local terminal** — spawn the host shell (PowerShell, bash, zsh) in a tab without an SSH connection.
- **Encrypted vault** — Argon2id KDF + AES-256-GCM per credential, plus SQLCipher full-DB encryption (defense in depth) with HKDF-split keys.
- **SFTP & port forwarding** — multi-panel SFTP browser with progress events; local (`-L`) and remote (`-R`) forwarding rules per host.
- **TOFU host verification** — fingerprint-on-first-use with mismatch dialog before any data is sent.
- **Encrypted multi-device sync** — payload-level AES-256-GCM with version-vector conflict resolution. Backends: HTTP (self-hosted), S3 (AWS Sig V4), WebDAV (Nextcloud / ownCloud / generic). Optional self-hosted Docker sync server included.
- **P2P LAN sync** — mDNS device discovery and PIN-paired peer-to-peer transfer for on-network sync without a backend.
- **VIP admin access** — generate device-bound OpenSSH keys for passwordless re-entry to trusted hosts.
- **Realtime server stats** — CPU, memory, disk, and uptime sampled per session.
- **Audit log** — hash-chained, encrypted, per-host opt-in with redaction and configurable retention.
- **Encrypted export/import** — SMEX format (Argon2id + AES-256-GCM) for offline backups and migration.
- **Plugin sandbox** — Wasmtime runtime for capability-gated WASM modules (no-import core, host APIs in progress).
- **Mobile-ready** — adaptive layout, bottom navigation, extended key bar, safe-area handling, Android target wired.
- **Theming** — three built-in themes (Dark / Light / High Contrast) plus user-defined themes via CSS variables.
- **Privacy by default** — no analytics, no third-party servers in the connection path. Your servers, your keys, your machine.

## Architecture

```
+----------------------------------------------------+
|              Frontend (React + WebView)            |
|  Terminal | Hosts | SFTP | Settings | Sync | Stats |
+--------------------------+-------------------------+
                           | invoke / events
+--------------------------v-------------------------+
|                Backend (Rust + Tauri v2)           |
|  ssh | sftp | port_forward | local_shell | vault   |
|  crypto | sync (HTTP/S3/WebDAV/P2P) | audit        |
|  plugin | team | known_hosts | server_stats        |
+--------------------------+-------------------------+
                           |
            +--------------+----------------+
            |                               |
       SQLite + SQLCipher          Remote SSH / SFTP
            |
            +-- E2E-encrypted sync to user's cloud
                (HTTP / S3 / WebDAV / P2P LAN)
```

Credentials never cross the IPC boundary. The React frontend works only with opaque host IDs; plaintext passwords and private keys exist only in Rust memory and are zeroized on drop.

```
Master Password
      | Argon2id
Master Key (256-bit)
      | HKDF (domain-separated)
   +--+---+
   |      |
Vault Key  DB Key (SQLCipher)
   |
AES-256-GCM per credential
```

## Tech Stack

| Layer | Technology |
|-------|------------|
| Shell | [Tauri v2](https://v2.tauri.app/) (desktop + mobile, native WebView) |
| Frontend | React 18 + Vite 6 + TypeScript (strict) |
| Styling | Tailwind CSS 3 with CSS-variable theme tokens |
| Terminal | [xterm.js](https://xtermjs.org/) + addon-fit / addon-search / addon-web-links |
| State | [Zustand](https://github.com/pmndrs/zustand) |
| SSH / SFTP | [`russh`](https://crates.io/crates/russh), [`russh-sftp`](https://crates.io/crates/russh-sftp) |
| Local PTY | [`portable-pty`](https://crates.io/crates/portable-pty) |
| Storage | SQLite via [`rusqlite`](https://crates.io/crates/rusqlite) with `bundled-sqlcipher` |
| Crypto | `aes-gcm`, `argon2`, `hkdf`, `hmac`, `sha2`, `ed25519-dalek`, `zeroize` |
| Sync | `reqwest` + custom AWS Sig V4 + WebDAV client |
| P2P | [`mdns-sd`](https://crates.io/crates/mdns-sd) for LAN discovery |
| Plugins | [Wasmtime v29](https://wasmtime.dev/) (component model + WASI) |

## Repository Layout

```
shellmate/
├── src/                       React frontend
│   ├── components/            UI (terminal, hosts, sftp, settings, sync, vip, server, mobile)
│   ├── hooks/                 useAutoLock, useIsMobile, useKeyboardShortcuts, ...
│   ├── stores/                Zustand stores (host, tab, vault, sftp, sync, broadcast, ...)
│   ├── lib/                   Typed Tauri invoke wrappers, snippet expansion
│   ├── types/                 Shared TS types (host, ssh, sftp, sync, plugin, audit, ...)
│   └── themes/                Built-in theme definitions
├── src-tauri/                 Rust backend
│   ├── src/commands/          IPC routes (ssh, sftp, vault, sync, p2p_sync, vip_access, ...)
│   ├── src/ssh/               Multi-session manager, PTY, broadcast, reconnect
│   ├── src/sftp/              russh-sftp subsystem manager
│   ├── src/port_forward/      Local & remote forwarding
│   ├── src/known_hosts/       TOFU verification
│   ├── src/vault/             Lock state machine, secure buffer
│   ├── src/crypto/            AES-GCM + Argon2id + HKDF primitives
│   ├── src/sync/              Sync engine + HTTP / S3 / WebDAV backends
│   ├── src/plugin/            Wasmtime runtime, manifest, capability checks
│   ├── src/audit/             Hash-chained, encrypted audit log
│   ├── src/db/                SQLite + SQLCipher schema and migrations
│   ├── Cargo.toml
│   └── tauri.conf.json
├── docs/                      Architecture, security, frontend, backend, devops plans
├── PRD.md                     Master product requirements
├── CHANGELOG.md
├── LICENSE                    MIT
└── README.md
```

## Getting Started

### Prerequisites

- [Rust](https://rustup.rs/) (MSVC toolchain on Windows)
- [Node.js](https://nodejs.org/) 18 or newer (npm ships with it)
- Platform-specific Tauri prerequisites — see the [Tauri v2 setup guide](https://v2.tauri.app/start/prerequisites/)

### Install & run

```bash
# Install frontend dependencies
npm install

# Run the desktop app (hot-reload)
npm run tauri:dev

# Type-check the frontend
npm run typecheck

# Lint
npm run lint
npm run lint:fix

# Format
npm run format
npm run format:check
```

### Production build

```bash
npm run tauri:build
```

Tauri produces installers under `src-tauri/target/release/bundle/` for the host platform. Code-signing certificates and the auto-updater public key are not committed; configure them through `tauri.conf.json` and your platform's signing toolchain before distributing.

### Mobile (Android)

The Rust crate is `cfg`-gated for Android (`jni`, `android_logger`, bundled `rusqlite`). Use the Tauri mobile CLI to build:

```bash
npm run tauri android init
npm run tauri android dev
npm run tauri android build
```

iOS targets compile against the same crate; an Xcode toolchain is required.

## Configuration

ShellMate stores all state under the OS user-data directory (resolved via the [`dirs`](https://crates.io/crates/dirs) crate). Nothing is read from environment variables at runtime.

| Subsystem | Where |
|-----------|-------|
| Encrypted DB | `<user_data>/shellmate/shellmate.db` (SQLCipher) |
| Known hosts | Stored in the encrypted DB |
| Audit log | Stored in the encrypted DB |
| Sync metadata | Stored in the encrypted DB |
| Plugin manifests | `<user_data>/shellmate/plugins/` |

The vault must be initialized with a master password on first launch. Recovery is intentionally impossible — the master password is the only way to decrypt the database, and the onboarding flow requires explicit acknowledgement of this.

## Security Model

- **Argon2id** key derivation with OWASP-aligned parameters.
- **HKDF domain separation** of the master key into vault key and SQLCipher key.
- **AES-256-GCM** authenticated encryption for every credential, sync payload, and audit entry.
- **SQLCipher** full-DB encryption protects metadata that would otherwise leak (host names, group structure, history).
- **Zeroized memory** for keys and secret buffers (`zeroize` crate, `Drop` impls).
- **No-recovery vault** — gated behind a mandatory acknowledgement on first setup.
- **TOFU host keys** with fingerprint-mismatch warning before reuse.
- **Hash-chained audit log** — SHA-256 chain over encrypted events for tamper detection.
- **Plugin sandbox** — Wasmtime traps caught via `spawn_blocking`; capabilities are opt-in and validated before execution.
- **No telemetry** — outbound network traffic is limited to your SSH targets, your sync backend, and (when enabled) the Tauri updater endpoint you configure.

For the full threat model see [`docs/07-security-plan.md`](docs/07-security-plan.md).

## Documentation

| Document | Description |
|----------|-------------|
| [`PRD.md`](PRD.md) | Product requirements and decisions |
| [`docs/02-project-scope.md`](docs/02-project-scope.md) | v1.0 scope and non-functional targets |
| [`docs/06-architecture-plan.md`](docs/06-architecture-plan.md) | System architecture |
| [`docs/07-security-plan.md`](docs/07-security-plan.md) | Threat model, encryption, sandboxing |
| [`docs/architecture.md`](docs/architecture.md) | P2P LAN sync and VIP access |
| [`CHANGELOG.md`](CHANGELOG.md) | Release history |

## Roadmap

- Biometric unlock with OS-protected key wrapping (Windows Hello, Touch ID, Android BiometricPrompt).
- Team vault encrypted sharing (public-key wrapping, member revocation, key rotation).
- Plugin host APIs and WASI surface.
- Cross-platform CI/CD, code signing, and signed auto-updater feed.
- Mosh transport for unreliable mobile networks.

See the issue tracker on GitHub for current work.

## Contributing

Issues and pull requests are welcome. Before opening a PR:

1. `npm run typecheck && npm run lint`
2. Build the desktop app locally (`npm run tauri:dev`) to confirm the change runs.
3. Keep changes focused — features and refactors should ship in separate PRs.

## License

[MIT](LICENSE) — free to use, modify, and distribute.
