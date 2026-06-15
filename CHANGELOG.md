# Changelog

All notable changes to ShellMate will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added — Local Terminal & Premium Server Monitor (2026-06-15)

**VIP Administrator Access & Local Terminal** (`src-tauri/` & `src/`):
- Added `as_admin` parameter to `vip_inject_authorized_keys` and `vip_create_localhost_host` to configure elevated terminal access.
- Implemented Windows UAC elevation via temporary PowerShell script with strict ACLs (`SYSTEM` & `Administrators` only).
- Implemented Unix root elevation using native AppleScript (`osascript`) and Linux `pkexec` agents.
- Resolved Tauri `state not managed for field db` bug by passing `State<'_, AppState>` instead of raw database connection references.
- Added a prominent **"Open Local Terminal"** button in `Sidebar.tsx` to instantly launch localhost terminal session (with fallback to the VIP Access setup screen).
- Added an interactive **"Next Steps & How to Use"** guide card inside `VipAccessPanel.tsx` upon successful VIP setup.

**Real-time Server Monitor** (`src/`):
- Replaced the static, refresh-only stats screen with an auto-updating dashboard polling every 2 seconds.
- Implemented a High-DPI canvas-based `CanvasGraph` component with custom grid overlays, glow paths, and smooth gradients.
- Created live monitoring charts for **CPU Load**, **Memory Usage**, **Network Download (Rx)**, and **Network Upload (Tx)** speeds.
- Implemented network speed calculation based on cumulative transfer rate deltas.
- Polished layouts for Disks and Top Processes tables.
- Fixed TypeScript compiler types (`KeyItem`) for custom keyboard layouts in `MobileKeyBar.tsx`.

### Added — Phase 14: Polish & Distribution (2026-06-13)

**Toast Notifications** (`src/`):
- Zustand toast store with auto-dismiss (success/error/warning/info)
- `ToastContainer` component with typed styling, dismiss button, fixed top-right
- Integrated into AppLayout (desktop + mobile)

**Encrypted Export/Import** (`src-tauri/`):
- `export_hosts_encrypted` — exports all hosts with credentials as encrypted base64 blob (SMEX format)
- `import_hosts_encrypted` — decrypts and imports hosts, creates groups as needed
- Argon2id + AES-256-GCM encryption with separate export password
- Frontend wrappers: `tauri.export.hostsEncrypted()` / `importHostsEncrypted()`

### Added — Phase 13: Audit Log (2026-06-13)

**Audit System** (`src-tauri/`):
- `AuditLog` with hash-chained encrypted event storage
- Event types: session_start, session_end, sftp_transfer, command_sent, vault_lock/unlock, host CRUD, settings_changed
- AES-256-GCM encrypted payloads with vault key
- SHA256 hash chain for tamper detection
- Per-host opt-in (default OFF), command history separate opt-in
- Pattern-based redaction (password, token, api_key, etc.)
- Configurable retention per host (default 90 days)
- `purge()` removes events older than retention threshold
- JSONL export format
- Database tables: `audit_events`, `audit_settings` (migration 009)
- 6 Tauri commands: `audit_record`, `audit_query`, `audit_export`, `audit_purge`, `audit_get_settings`, `audit_set_settings`

**Frontend** (`src/`):
- `AuditEvent`, `AuditSettings`, `AuditQuery` types
- `tauri.audit.*` command wrappers

### Added — Phase 12: Plugin System (2026-06-13)

**Plugin Runtime** (`src-tauri/`):
- Wasmtime v29 WASM runtime with WASI support
- `PluginRuntime` — sandboxed execution, crash isolation via `spawn_blocking`
- `PluginManager` — install, list, uninstall, enable/disable, capability management
- `PluginManifest` — JSON manifest format with capability declarations + validation
- Capability model: `log`, `panel`, `terminal_data`, `network`, `filesystem`, `secrets` (all opt-in)
- Database tables: `plugins`, `plugin_capabilities` (migration 008)
- WASM files copied to `<app_data>/plugins/` on install, cleaned on uninstall
- 9 Tauri commands: `plugin_list`, `plugin_install`, `plugin_uninstall`, `plugin_enable`, `plugin_disable`, `plugin_get_capabilities`, `plugin_grant_capability`, `plugin_revoke_capability`, `plugin_execute`

**Frontend** (`src/`):
- `Plugin`, `PluginCapability`, `PluginManifest` types
- `tauri.plugin.*` command wrappers

**Dependencies**: `wasmtime` v29, `wasmtime-wasi` v29

### Added — Phase 11: Team Vault (2026-06-13)

**Team Management** (`src-tauri/`):
- `TeamManager` with full CRUD: create, list, delete teams
- Member management: add member by public key, list members, revoke (timestamp-based)
- Host sharing: share host with team, set permissions (read/edit), remove share
- Team master key: random 32 bytes, wrapped with vault key via AES-256-GCM
- Member key wrapping: HKDF-SHA256 derived key from member pubkey, AES-256-GCM wrap
- Database tables: `team`, `team_members`, `team_shares` with cascade deletes (migration 007)
- 9 Tauri commands: `team_create`, `team_list`, `team_delete`, `team_add_member`, `team_list_members`, `team_revoke_member`, `team_share_host`, `team_list_shares`, `team_remove_share`

**Frontend** (`src/`):
- `Team`, `TeamMember`, `TeamShare` types + input interfaces
- `tauri.team.*` command wrappers

### Added — Phase 10: Mobile Apps (2026-06-13)

**Mobile UI** (`src/`):
- `useIsMobile` hook — responsive mobile detection via `matchMedia` (768px breakpoint)
- `MobileLayout` — mobile-first layout with bottom navigation, StatusBar, VaultGate
- `BottomNav` — fixed bottom tab bar (Hosts, Terminal, Snippets, Settings)
- `MobileKeyBar` — extended key bar for terminal: Esc, Tab, Ctrl, Alt, arrows, symbols; Ctrl/Alt modifier toggle
- `AppLayout` — conditional render: MobileLayout on mobile, DesktopLayout on desktop
- Terminal component — flex layout with MobileKeyBar integration on mobile
- CSS safe area utilities for iOS notch/home indicator
- Touch scroll optimization and hidden scrollbars on coarse pointers
- `100dvh` viewport height fix for mobile browsers

**Android Config** (`src-tauri/`):
- `jni` v0.21 + `android_logger` v0.13 (cfg-gated to Android target)

### Added — Phase 9: Multi-Device Sync (E2E) (2026-06-13)

**Sync Engine** (`src-tauri/`):
- `SyncEngine` with version vector clocks for multi-device conflict detection
- Two-phase sync: upload pending changes, download remote changes
- Per-entity sync state tracking (`sync_state` table) with version vectors
- Sync backend configuration (`sync_config` table) with encrypted credentials
- `mark_changed()` API for tracking local mutations

**Sync Backends** (`src-tauri/src/sync/backend/`):
- `SyncBackend` trait with async `put/get/list/delete` methods
- `HttpBackend` — self-hosted HTTP REST API with bearer token auth
- `S3Backend` — AWS Signature V4 signing, supports AWS S3, MinIO, Backblaze B2
- Opaque UUID object IDs — no metadata leakage in cloud storage keys

**Encryption & Conflict Resolution**:
- AES-256-GCM per-payload encryption with HKDF-derived key (nonce prepended)
- Version vector conflict detection (concurrent edit detection)
- Last-write-wins conflict resolution

**Commands**: `sync_status`, `sync_configure`, `sync_now`, `sync_pause`, `sync_resume`

**Frontend** (`src/`):
- `SyncStatus`, `SyncConfigureInput`, `SyncResult` types
- `tauri.sync` command wrappers (status/configure/now/pause/resume)

**Dependencies**: `reqwest` v0.13, `hmac` v0.12

### Added — Phase 8: Biometric Unlock (2026-06-13)

**Windows Hello Integration** (`src-tauri/`):
- Biometric unlock via Windows Hello `KeyCredentialManager` (TPM-backed key creation/opening)
- `BiometricProvider` trait with platform-specific dispatch (`cfg(target_os)`)
- AES-256-GCM vault key wrapping with HKDF-derived key from 32-byte device secret
- `biometric_state` SQLite table for per-device enrollment persistence
- `vault.unlock_with_key()` for direct key-based unlock (bypasses password verification)
- Commands: `biometric_status`, `biometric_enable`, `biometric_disable`, `biometric_unlock`
- Biometric failures independent of master password lockout counter
- `windows` v0.62 + `windows-future` v0.3 dependencies (Windows-only, cfg-gated)

**Frontend** (`src/`):
- `BiometricStatus` type (available, enabled, platform)
- `tauri.biometric` command wrappers (status/enable/disable/unlock)

### Added — Phase 7: Full-DB Encryption (SQLCipher) (2026-06-13)

**SQLCipher Integration** (`src-tauri/`):
- Full database encryption at rest using SQLCipher (AES-256-CBC + HMAC-SHA512)
- `rusqlite` upgraded to `bundled-sqlcipher` feature for compile-time SQLCipher bundling
- HKDF-SHA256 key derivation with domain separation: vault key (`info: "vault-key"`) and DB key (`info: "db-key"`) derived from single Argon2id master key
- Automatic plaintext-to-encrypted migration on first vault unlock (backup as `.db.bak`)
- `PRAGMA rekey` for in-place SQLCipher key rotation on master password change
- `AppState.swap_db()` for runtime connection replacement after migration
- `vault_status` command now includes `db_encrypted` field
- All master keys zeroized immediately after HKDF split; vault/db keys zeroized on lock or error

**Security Properties**:
- Defense in depth: per-credential AES-256-GCM + SQLCipher full-DB encryption
- Metadata protection: hostnames, usernames, groups, snippets, settings encrypted at rest
- Key isolation: vault key and DB key derived via HKDF with different `info` parameters
- Zeroization: master key zeroized after split; keys zeroized on lock/error

**Build Dependencies**:
- OpenSSL dev headers required for `bundled-sqlcipher` on Windows (`OPENSSL_LIB_DIR`, `OPENSSL_INCLUDE_DIR`)

### Added — Termul Feature Parity & Bug Fixes (2026-06-13)

**Critical Security Fixes** (`src-tauri/`):
- P2P sync: replaced single SHA-256 PIN key derivation with Argon2id (memory-hard), bound server to `127.0.0.1` instead of `0.0.0.0`, added per-IP rate limiting (10 attempts/60s window).
- VIP access: fixed `inject_authorized_keys` to convert hex-encoded public key to base64 (required by OpenSSH `authorized_keys` format).

**Bug Fixes** (`src/`):
- Fixed SFTP progress listener race condition — now `await`s `openBrowser()` before subscribing to events.
- Terminal now reads settings from `useSettingsStore` (fontSize, scrollback, cursorStyle, cursorBlink, theme) instead of hardcoded values.
- Broadcast mode now uses SSH session IDs (via `ssh-store.sessionByTab`) instead of tab IDs.
- Replaced non-existent shadcn/ui CSS classes in VipAccessPanel and P2pSyncPanel with project Tailwind classes.
- Deleted stale `vite.config.js` (compiled artifact duplicate).

**Backend Fixes** (`src-tauri/`):
- `commands/known_hosts.rs` and `commands/broadcast.rs`: changed `Result<T, String>` → `AppResult<T>` for consistency.
- `port_forward/mod.rs`: fixed `toggle_forward` to actually pause/resume TCP listener via shared `Arc<RwLock<bool>>`.
- `commands/host.rs`: added SQL LIKE wildcard escaping (`%`, `_`) in `search_hosts`.

**Documentation Fixes**:
- `docs/08-devops-plan.md`: replaced all `bun` references with `npm`.
- `docs/02-project-scope.md`: fixed NFR targets to match PRD, updated Mosh to Phase 14.
- `README.md`: fixed Mosh phase reference.

**New Features — Power User Tools**:
- **Error Boundaries**: `ErrorBoundary` component wrapping app at top and feature levels with fallback UI and reset.
- **Command Palette** (`Ctrl+K` / `Ctrl+Shift+P`): modal with fuzzy search, grouped results, keyboard navigation, 13 default commands.
- **Keyboard Shortcuts System**: store + hook with 6 default bindings (Ctrl+T, Ctrl+W, Ctrl+B, Ctrl+K, Ctrl+PageDown/Up).
- **Git Integration**: StatusBar shows local git branch, dirty state, ahead/behind indicators (polls every 10s via `git_get_info` Tauri command).
- **Multiple Shell Support**: optional shell field on QuickConnect (bash, zsh, fish, sh, PowerShell, CMD); uses `channel.exec()` for custom shell.
- **Command History**: `command_history` DB table, CRUD commands, HistoryPanel with search and click-to-rerun.
- **Auto-Updater**: `tauri-plugin-updater` integrated with 30-min check interval, UpdateToast with download+install+relaunch.

### Fixed — Phase 1–6 Integration & Compile-Time Fixes (2026-06-11)

**Backend** (`src-tauri/`):
- Upgraded `russh-sftp` dependency version from `"0.4"` to `"2.1.2"` to resolve crates.io version mismatch.
- Refactored `sftp/mod.rs` to store `Arc<tokio::sync::Mutex<SftpSessionWrapper>>` to avoid holding synchronous parking_lot Mutex guards across async `.await` boundaries.
- Wired SSH connection parameter and active handle registration upon successful client authentication in `SessionManager::open`.
- Removed invalid top-level `mod broadcast;` from `lib.rs` and registered missing broadcast Tauri commands.
- Added `#[serde(rename_all = "camelCase")]` and explicit serde renames for TOFU host-key verification struct responses to align with frontend Tauri commands.
- Added `sha2` crate dependency for cryptographic fingerprint hashing.

**Frontend** (`src/`):
- Registered global event listener for `"ssh:host-key-verification"` in `AppLayout.tsx` to display `HostKeyVerificationDialog` properly during TOFU connection handshakes.
- Refactored connection attempt management and retries within `ssh-store.ts`.

### Added — Phase 6: Network Hardening (2026-06-10)

**Backend** (`src-tauri/`):
- `db/schema.rs` migration `003_known_hosts` adds `known_hosts` table (id, hostname, port, key_type, fingerprint, public_key_blob, trusted, timestamps)
- `known_hosts/mod.rs` — TOFU (Trust On First Use) host key verification:
  - `KnownHostsManager` with SHA256 fingerprint calculation
  - `verify_host_key` — checks if host is known and trusted, detects mismatches
  - `trust_host_key` — adds/updates known host entries
  - `list_known_hosts`, `remove_host_key`, `set_trusted` — management operations
- `ssh/handler.rs` updated to use TOFU verification:
  - Extracts key type and blob from server public key
  - Verifies against known_hosts database
  - Auto-trusts new hosts (TOFU), rejects mismatches with security warning
- `ssh/reconnect.rs` — auto-reconnect with exponential backoff:
  - `ReconnectHandle` with cancellation support
  - Backoff schedule: 1s, 2s, 5s, 10s, 30s, 60s (max)
  - Emits status events: Waiting, Connecting, Connected, Failed, Cancelled
  - Re-authenticates on successful reconnection
- `ssh/broadcast.rs` — multi-tab keystroke broadcasting:
  - `BroadcastManager` tracks active broadcast sessions
  - `broadcast_input` sends data to all subscribed sessions
  - `subscribe` returns broadcast receiver for session forwarding
- `ssh/session.rs` updated to pass `known_hosts` manager to `ClientHandler`
- `state.rs` extended with `known_hosts: Arc<KnownHostsManager>` and `broadcast: Arc<BroadcastManager>`
- `commands/known_hosts.rs` — Tauri commands: `known_hosts_verify`, `known_hosts_trust`, `known_hosts_list`, `known_hosts_remove`, `known_hosts_set_trusted`
- `commands/broadcast.rs` — Tauri commands: `broadcast_add`, `broadcast_remove`, `broadcast_is_active`, `broadcast_get_sessions`, `broadcast_send`
- All commands wired into `lib.rs`

**Frontend** (`src/`):
- Types:
  - `types/known-hosts.ts` — `KnownHost`, `HostKeyVerificationResult`, input types
  - `types/broadcast.ts` — `BroadcastSession`
- Stores:
  - `stores/broadcast-store.ts` — Zustand store for broadcast session management, loadSessions, addSession, removeSession, isSessionActive, sendToAll
- `lib/tauri.ts` extended with `knownHosts` and `broadcast` command wrappers
- Components:
  - `components/security/HostKeyVerificationDialog.tsx` — modal dialog for new host keys and mismatch warnings, shows fingerprints, trust/reject actions
  - `components/terminal/BroadcastModePanel.tsx` — session selection UI with checkboxes, command input field, send to all selected sessions, visual indicators for active broadcasts
- UI integration:
  - `stores/ui-store.ts` extended with `'broadcast'` in `ActivePanel` union
  - `components/layout/TabBar.tsx` shows Broadcast button (Radio icon) when session is active
  - `components/layout/ContentArea.tsx` renders `BroadcastModePanel` when `activePanel === 'broadcast'`
- i18n strings extended for `hostKeyVerification.*`, `broadcast.*`, and `reconnect.*`

**Features Implemented:**
- ✅ Known hosts table populated on first connect (TOFU)
- ✅ Verification UI: show fingerprint, ask user to trust
- ✅ On key mismatch: warning dialog with old vs new fingerprint, options (trust new, abort)
- ✅ Auto-reconnect: exponential backoff (1s, 2s, 5s, 10s, 30s, 60s max)
- ✅ User-cancellable reconnect with status visible in tab
- ✅ Broadcast mode: select multiple tabs, single input field broadcasts keystrokes
- ✅ Visual indicator on broadcasted tabs

### Added — Phase 5: File Transfer & Network (2026-06-10)

**Backend** (`src-tauri/`):
- `sftp/mod.rs` — SFTP subsystem support via `russh-sftp`:
  - `SftpManager` manages multiple SFTP sessions per SSH connection
  - Operations: `list_directory`, `upload_file`, `download_file`, `rename`, `remove`, `mkdir`
  - Progress tracking with events for transfers > 1 MB
  - Automatic file type detection (directory, symlink, regular file)
  - Sorts files with directories first, then alphabetical
- `port_forward/mod.rs` — Local and remote port forwarding:
  - `PortForwardManager` manages active forwards per session
  - `create_forward` binds local port and spawns SSH channel for TCP forwarding
  - `toggle_forward` enables/disables rules without disconnecting
  - Automatic cleanup on session disconnect
  - Port conflict detection (fails with clear error if port bound)
- `commands/sftp.rs` — Tauri commands: `sftp_open`, `sftp_list`, `sftp_upload`, `sftp_download`, `sftp_rename`, `sftp_remove`, `sftp_mkdir`, `sftp_close`
- `commands/port_forward.rs` — Tauri commands: `port_forward_create`, `port_forward_list`, `port_forward_remove`, `port_forward_toggle`
- `state.rs` extended with `SftpManager` and `PortForwardManager`
- `lib.rs` updated to include `sftp` and `port_forward` modules
- All commands wired into `lib.rs` invoke handler

**Frontend** (`src/`):
- Types: `types/sftp.ts` (`SftpFile`, `SftpProgressEvent`), `types/port-forward.ts` (`PortForwardRule`, `PortForwardType`)
- Stores:
  - `stores/sftp-store.ts` — manages multiple SFTP browser instances, file operations, transfer progress tracking
  - `stores/port-forward-store.ts` — manages port forward rules per session, CRUD operations
- `lib/tauri.ts` extended with `sftp` and `portForward` command wrappers
- Components:
  - `components/sftp/SftpBrowser.tsx` — file browser with breadcrumbs, toolbar (upload, mkdir, refresh), file table with download/delete actions, transfer progress indicators
  - `components/port-forward/PortForwardPanel.tsx` — rule creation form (local/remote toggle, port validation), rule list with enable/disable toggles
- UI integration:
  - `stores/ui-store.ts` extended with `ActivePanel` types for `'sftp'` and `'port-forward'`, plus `sftpSessionId` and `portForwardSessionId` state
  - `components/layout/ContentArea.tsx` renders `SftpBrowser` and `PortForwardPanel` when active
  - `components/layout/TabBar.tsx` shows SFTP and Port Forwarding buttons when a session is active

**Dependencies** (`src-tauri/Cargo.toml`):
- Added `russh-sftp = "0.4"` for SFTP protocol support
- Added `bytes = "1"` for efficient binary data handling

**Features Implemented:**
- ✅ SFTP browser opens as panel within active session
- ✅ Directory listing: name, size, permissions, modified date, file type icon
- ✅ Navigation: up, into directory, breadcrumb, address bar
- ✅ Operations: upload, download, rename, delete, mkdir
- ✅ Progress indicator for transfers (real-time percentage)
- ✅ Multiple SFTP windows per session supported
- ✅ SFTP runs as separate channel on parent SSH connection
- ✅ Port forwarding: local (-L) and remote (-R) rules per host
- ✅ Toggle rule on/off without disconnecting
- ✅ Conflict detection: clear error if local port already bound

### Added — Phase 4: Productivity & Settings (2026-06-10)

**Backend** (`src-tauri/`):
- `db/schema.rs` migration `002_themes` adds `themes` table (id, name, base, definition JSON, is_builtin flag, timestamps)
- `commands/snippet.rs` — full CRUD: `get_snippets`, `create_snippet`, `update_snippet`, `delete_snippet` with title+command validation, JSON-serialized tags
- `commands/theme.rs` — `get_themes`, `save_theme` (UPSERT for custom only, rejects modifying builtins), `delete_theme` (rejects builtins)
- `vault/mod.rs::change_master_password` — atomic re-encryption:
  - Verifies current password via constant-time compare against verifier blob
  - Derives new key with fresh salt (via `Argon2id`)
  - Single SQLite transaction: re-encrypts every credential, replaces verifier blob, replaces stored salt
  - Zeroizes both old and new keys on every error path
  - Swaps in-memory key only after `tx.commit()` succeeds
- `commands/vault.rs::vault_change_master_password` Tauri command
- All commands wired into `lib.rs`

**Frontend** (`src/`):
- Types: `types/snippet.ts`, `types/theme.ts` (`ThemeBase`, `ThemeDefinition`, `Theme`, `ThemeInput`)
- Theme system:
  - `themes/builtin.ts` — 3 ThemeDefinition objects (ShellMate Dark, ShellMate Light, High Contrast)
  - `applyTheme()` sets CSS variables on `<html>` + toggles Tailwind `dark` class
  - `tailwind.config.js` rewritten to read all colors from `var(--color-*)`
  - `styles/globals.css` provides default `:root` variables (dark theme baseline)
- Stores:
  - `stores/snippet-store.ts` — load/add/update/remove + searchQuery
  - `stores/settings-store.ts` — full settings state (themeId, fontSize, scrollback, autolockSecs, cursorStyle, cursorBlink) + theme save/delete + `resolveTheme(id)` helper
- Lib:
  - `lib/snippet-expand.ts` — `expandSnippet`, `extractPlaceholders`, `unknownPlaceholders`
  - `lib/tauri.ts` extended for `snippets`, `themes`, `vault.changeMasterPassword`
- Hook:
  - `hooks/useAutoLock.ts` — polls `vault_check_idle` every 15s; throttled activity ping every 60s on user input (mousedown/keydown/wheel/touchstart)
- Components:
  - `components/snippets/SnippetForm.tsx` — modal CRUD, multiline command, tag parsing
  - `components/snippets/SnippetPanel.tsx` — list/search/execute UI; warns when no active session or unknown placeholders detected; sends `command\n` to active session via `tauri.ssh.send`
  - `components/settings/SettingsDialog.tsx` — tabbed dialog (General/Terminal/Vault/Theme)
  - `components/settings/GeneralSettingsTab.tsx` — app/version/license info
  - `components/settings/TerminalSettingsTab.tsx` — font size, scrollback, cursor style, blink
  - `components/settings/VaultSettingsTab.tsx` — autolock dropdown, "Lock now" button, master password change form with current/new/confirm + validation + success message
  - `components/settings/ThemeSettingsTab.tsx` — theme grid with terminal palette swatch preview, apply/delete actions; built-ins non-deletable
- Wiring:
  - `Sidebar.tsx` — Snippets/Settings panel buttons toggle activePanel back to 'hosts' when clicked again
  - `ContentArea.tsx` — renders `SnippetPanel` for activePanel='snippets'; renders `SettingsDialog` modal for activePanel='settings'
  - `App.tsx` — loads `settings-store` on mount (before vault), wires `useAutoLock` hook
- i18n strings extended for `snippets.*` and `settings.*`

### Verified
- `npm run typecheck` — pass
- `npm run lint` — pass
- `npm run format:check` — pass
- `npm run build` — pass (559 KB / 154 KB gzipped, within 500 KB gzipped budget)
- `cargo build` — pass (incremental 0.76s, 8 forward-compat warnings unchanged)

### Decided During Phase 4
- **Theme architecture**: CSS variables on `<html>`, Tailwind reads via `var(--color-*)`. All existing components retheme automatically. Toggle `dark` class for Tailwind dark variant compatibility.
- **3 built-in themes**: ShellMate Dark (original), ShellMate Light (inverted with same hue), High Contrast (WCAG AAA-ready, yellow-on-black).
- **Custom theme editor UI**: backend + storage shipped (user can call `tauri.themes.save` programmatically); full color-picker editor UI **deferred to Phase 14 polish**.
- **Configurable keyboard shortcuts**: deferred to Phase 14. Default Phase 1 shortcuts remain hardcoded for now.
- **Master password change atomicity**: single SQLite transaction; rollback on any error; both keys zeroized on every failure path.
- **Auto-lock policy**: frontend polls 15s rather than backend pushing events. Activity ping throttled to once per 60s.
- **Settings storage**: key-value rows in existing `settings` table with namespace prefixes (`ui.theme.id`, `terminal.font_size`, etc.).
- **Snippet placeholder strategy**: built-in vars auto-expand from active host (host, username, port, label). Unknown vars trigger UI warning before execute — prevents footguns.

### Carried over to Phase 14
- Tag autocomplete on HostForm
- Markdown notes preview
- Configurable keyboard shortcuts editor
- Custom theme color-picker editor

---

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
