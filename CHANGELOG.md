# Changelog

All notable changes to ShellMate will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added — Phase 3: Host Management & Persistence (2026-06-10)

**Backend** (`src-tauri/src/commands/`):
- `group.rs` — full CRUD: `get_groups`, `create_group`, `update_group`, `delete_group`, `move_host_to_group`
  - Validation: name required, hex color check (#RGB or #RRGGBB), prevent self-parent cycles
  - Delete cascades: hosts in deleted group become ungrouped; sub-groups become detached
- `host.rs` extended with `search_hosts` — case-insensitive multi-field LIKE query joining `hosts` and `groups` (label, hostname, username, group name, tags, notes)
- All commands wired into `lib.rs` invoke handler

**Frontend** — UI primitives (`src/components/ui/`):
- `Button.tsx` — 4 variants (primary, secondary, ghost, danger) × 2 sizes (sm, md)
- `Form.tsx` — `Input`, `Textarea`, `Select`, `Field` (label + error + hint pattern with ARIA)
- `Modal.tsx` — focus trap, Esc-to-close, click-outside-to-close, accessible (role="dialog", aria-modal, labelledby)
- `ConfirmDialog.tsx` — destructive action confirmation

**Frontend** — Host management (`src/components/hosts/`):
- `HostForm.tsx` — add/edit modal with full validation, password and SSH key auth, edit mode keeps existing credential when password field blank
- `HostItem.tsx` — sidebar row with drag-and-drop, right-click context menu (Connect/Edit/Delete), double-click and Enter to connect, group color dot, hostname tooltip
- `HostList.tsx` — grouped sections, expand/collapse, drop target visual feedback, empty + no-results states, search-driven force-expand
- `GroupForm.tsx` — modal with 6 preset color swatches + custom hex input

**Frontend** — Stores & wiring:
- `stores/host-store.ts` rewritten with full state (hosts, groups, search, expandedGroups) + actions for both entity types
- `lib/tauri.ts` extended with `groups` and `hosts.search/moveToGroup`
- `types/host.ts` adds `GroupInput`
- `Sidebar.tsx` rewritten to load real data on vault unlock, wire search input, render `HostList`, add "Add Host" + "New Group" footer buttons
- New i18n strings under `hostForm.*`, `groupForm.*`, `hostActions.*`

### Verified
- `npm run typecheck` — pass
- `npm run lint` — pass
- `npm run format:check` — pass
- `npm run build` — pass (533 KB / 147 KB gzipped, within 500 KB gzipped budget)
- `cargo build` — pass (incremental 0.73s, 8 forward-compat warnings)

### Decided During Phase 3
- **Search strategy**: client-side filter (data is small, ms response). Backend `search_hosts` is shipped for future server-side use cases (audit join etc).
- **Drag-and-drop**: native HTML5 with `application/x-shellmate-host` MIME — no external library.
- **Modal lib**: lightweight in-house Modal (~50 lines) for now. Will evaluate swap to shadcn/ui Dialog in Phase 4 when more dialog patterns appear.
- **Group nesting**: schema supports `parent_id` (Phase 1), backend rejects self-parent. UI is flat — tree visualization deferred to Phase 4 if needed.
- **Tag autocomplete + markdown notes preview**: deferred to Phase 4 (basic comma-separated input + plain textarea shipped).
- **Credential lifecycle on edit**: blank credential field means "keep existing"; non-blank creates a new credential row. Old row not auto-deleted on host edit (could be referenced); deleted on host delete.
- **Connect flow**: click connect / Enter / double-click → frontend tab + `ssh_connect(host_id)`. Existing Phase 2 Terminal subscribes to events automatically.

---

### Changed — Scope Expansion to v1.0 Production (2026-06-10)

**Project repositioned from "MVP" to "v1.0 production release."** All planning documents updated to reflect the broader vision.

**Documents updated:**
- `PRD.md` → v2.0: tujuan utama termasuk multi-device sync, plugin system, team vault, biometric, audit, custom themes, broadcast mode, Mosh. Resolved Decisions table updated with v1.0 commitments. Milestones rewritten as 14 scope-driven phases (no fixed timeline).
- `README.md`: updated overview, feature list, tech stack table, roadmap.
- `docs/01-development-plan.md` → v2.0: scope-driven phases, Phase 1-2 marked complete, Phase 3-14 with explicit acceptance criteria.
- `docs/02-project-scope.md` → v2.0: dropped MVP framing, in-scope/out-of-scope rewritten for v1.0, success criteria expanded.
- `docs/06-architecture-plan.md` → v2.0: added §11 Multi-Platform, §12 Sync, §13 Plugin Architecture (Wasmtime), §14 Audit Log, §15 Theme System, §16 Broadcast Mode.
- `docs/07-security-plan.md` → v2.0: §11 Encryption Strategy flipped to defense-in-depth (per-credential AES-GCM + SQLCipher both active); added §12 Biometric Security, §13 Sync Security, §14 Team Vault Security, §15 Plugin Security, §16 Audit Log Security.
- `docs/08-devops-plan.md` → v2.0: §11 Code Signing now mandatory for v1.0; added §15 Mobile Build & Distribution, §16 Plugin Distribution, §17 Sync Backend Setup, §18 Security Audit Pipeline.
- `docs/04-backend-plan.md` → v2.0: §10 New Backend Modules (mosh, sync, plugin, audit, team, biometric, theme).
- `docs/03-frontend-plan.md` → v2.0: §11 Mobile UX Architecture, §12 Theme System, §13 Broadcast Mode UI, §14 Sync UI, §15 Plugin UI, §16 Audit Log UI, §17 Team Vault UI.
- `docs/05-erd-plan.md` → v2.0: Appendix A new tables (themes, known_hosts, sync_state, sync_config, audit_events, audit_settings, team, team_members, team_shares, plugins, plugin_capabilities, biometric_state).

**Key decisions made:**
- Encryption: defense-in-depth (per-credential AES-256-GCM + SQLCipher full-DB) — both layers active
- Code signing: required for v1.0 release (Apple, Authenticode, GPG)
- Auto-updater: required for v1.0 (Tauri v2 updater with signed releases)
- Sync architecture: user's own cloud only, E2E encrypted, no ShellMate server
- Plugin runtime: Wasmtime sandbox with capability-based permissions
- Mobile: Android first, iOS next
- Timeline: scope-driven (no fixed deadlines), each phase ships when acceptance criteria are met

**No code changes in this entry** — Phase 1-2 implementation remains valid; new modules will be added in Phase 3-14.

---

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
- TOFU client handler (known_hosts deferred to Phase 6)
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
- **Host key verification**: TOFU-accepting handler for now. Known_hosts table + verification UI in Phase 6.
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
- i18n string module (`src/i18n/en.ts`) — English default, ready for translation in later phases
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
