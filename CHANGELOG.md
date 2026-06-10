# Changelog

All notable changes to ShellMate will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added — Phase 2: Core SSH (2026-06-10)

**Crypto primitives** (`src-tauri/src/crypto/`):
- Argon2id KDF (64 MiB / 3 iter / 4 parallelism / 32-byte output) per OWASP guidance
- AES-256-GCM authenticated encryption with random 12-byte nonce per encryption
- `SecureBuffer` wrapper that zeroizes on drop; intentionally `!Clone` and `!Debug`
- Unit tests for roundtrip, wrong-key fail, tampered-ciphertext fail, deterministic derivation

**Vault** (`src-tauri/src/vault/`):
- Vault state machine (uninitialized → setup → unlocked ↔ locked)
- Master password verifier blob with constant-time comparison via `subtle::ct_eq`
- Length-first password policy (12-128 chars) per NIST SP 800-63B
- Idle auto-lock check (default 15 min)
- Manual lock zeroizes derived key

**Tauri commands**:
- Vault: `vault_status`, `vault_setup`, `vault_unlock`, `vault_lock`, `vault_check_idle`, `vault_record_activity`
- Credentials: `save_credential`, `delete_credential` (encrypted via vault key)
- SSH: `ssh_connect`, `ssh_quick_connect`, `ssh_send`, `ssh_resize`, `ssh_disconnect`

**SSH** (`src-tauri/src/ssh/`, russh 0.45):
- TOFU client handler (known_hosts deferred to Phase 4)
- `SessionManager` with one async task per session, mpsc command channel
- 1 SSH connection per tab strategy (per docs/04-backend-plan §9)
- PTY (xterm-256color), shell channel, keepalive (60s, max 3 retries)
- Per-session events: `ssh:output:{id}`, `ssh:status:{id}`, `ssh:error:{id}`
- Limits: SOFT_SESSION_LIMIT = 20, MAX_SESSIONS = 50
- Auth methods: password, private key (with optional passphrase)

**Frontend**:
- `vault-store` and `ssh-store` (Zustand)
- `VaultGate` — gates the entire app behind vault unlock
- `VaultSetup` with mandatory recovery warning + acknowledge checkbox (per docs/07-security §4.1.2)
- `VaultUnlock` with constant-time backend verification
- `Terminal` component (xterm.js + FitAddon + WebLinksAddon) with SSH event subscription and ResizeObserver
- `QuickConnect` form — one-off SSH session for testing (clears sensitive fields after submit)
- `ContentArea` keeps all terminals mounted with visibility toggling so xterm state survives tab switches
- `TabBar` cleanup: disconnects backend session and unbinds on tab close
- `StatusBar` lock button with disabled state when already locked
- Typed Tauri wrapper extended for vault, credentials, SSH

### Verified
- `npm run typecheck` — pass
- `npm run lint` — pass
- `npm run format:check` — pass
- `npm run build` — pass (509 KB / 140 KB gzipped, within 500 KB gzipped budget)
- `cargo build` — pass (8 forward-compat unused-API warnings)

### Decided During Phase 2
- **russh version**: pinned to 0.45 (older API: `authenticate_*` returns `bool`, `check_server_key` takes `key::PublicKey`). 0.50+ has breaking changes that require additional adapter work — deferred.
- **Host key verification**: TOFU-accepting handler for MVP. Known_hosts table + verification UI deferred to Phase 4.
- **Verifier scheme**: encrypted constant compared with `subtle::ct_eq`. AES-GCM auth tag provides integrity; no separate key hash needed.
- **No password recovery**: hardcoded into UX. Setup form blocks submit until user explicitly checks the acknowledgement.
- **xterm tab persistence**: `ContentArea` keeps all terminals mounted with `visibility: hidden` to preserve state across tab switches.
- **Disk space note**: cargo target dir grew to ~9 GB during Phase 2 builds. `cargo clean` recovered space. Worth tracking for Phase 3+.

---

### Added — Phase 1: Project Setup (2026-06-09)

- Tauri v2 project scaffold with React 18 + Vite 6 + TypeScript (strict mode)
- Tailwind CSS 3 with custom dark theme palette (bg, sidebar, panel, elevated, border, fg, accent, status colors)
- SQLite database with full schema migrations:
  - Tables: `groups`, `credentials`, `hosts`, `snippets`, `port_forwards`, `settings`
  - WAL journal mode, foreign keys enforced, parameterized queries
  - Migration runner with `_migrations` tracking table
- Tauri commands: `app_version`, `get_hosts`, `create_host`, `update_host`, `delete_host`, `get_settings`, `set_setting`
- Application state with `parking_lot::Mutex<Connection>`
- Serializable `AppError` type via `thiserror`
- Layout components:
  - `AppLayout` — root layout shell
  - `TitleBar` — custom title bar with drag region and window controls
  - `Sidebar` — host list with search, groups, action buttons
  - `TabBar` — multi-tab session bar with status indicators
  - `StatusBar` — vault state and active connection display with `aria-live`
  - `ContentArea` — placeholder for terminal/SFTP/settings panels
- Zustand stores:
  - `tab-store` — multi-tab session state
  - `ui-store` — sidebar, panel, vault state
  - `host-store` — host CRUD with backend sync
- Typed Tauri invoke wrapper (`src/lib/tauri.ts`)
- i18n string module (`src/i18n/en.ts`) — English default, ready for post-MVP translation
- ESLint + Prettier + tsconfig strict mode
- MIT LICENSE
- Tauri icon set generated from custom SVG logo
- `rust-toolchain.toml` pinned to MSVC stable

### Verified
- `npm run typecheck` — pass
- `npm run lint` — pass
- `npm run build` — pass (197 KB / 62 KB gzipped, well within 500 KB budget)
- `cargo build` — pass (1 ringan unused-variant warning untuk forward-compat error variant)

### Decided During Phase 1
- Package manager: **npm** (Bun on local system was POSIX-only binary)
- Rust toolchain: **MSVC** (mingw-gnu had "export ordinal too large" linker error with Tauri)
- Window decorations: disabled, custom title bar implementation
- Frontend bundle baseline: 62 KB gzipped — performance budget headroom for Phase 2-6
