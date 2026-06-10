# Development Plan
## ShellMate — SSH Client (Production v1.0)

**Version:** 2.1
**Last Updated:** 2026-06-10
**Approach:** Scope-driven phases, no fixed timeline — each phase ships when acceptance criteria are met.

---

## 0. Progress Tracker

| Phase | Status | Completed | Notes |
|-------|--------|-----------|-------|
| Phase 1: Project Setup | ✅ Complete | 2026-06-09 | Tauri v2 + React/Vite/TS scaffold, SQLite schema, layout shell, stores |
| Phase 2: Core SSH | ✅ Complete | 2026-06-10 | Vault (Argon2id + AES-256-GCM), russh integration, xterm terminal, multi-tab session manager |
| Phase 3: Host Management & Persistence | ✅ Complete | 2026-06-10 | Host CRUD UI, Group CRUD, drag-and-drop, host search, connect from sidebar |
| Phase 4: Productivity & Settings | ⏳ Pending | — | Snippets, settings, custom themes, configurable shortcuts |
| Phase 5: File Transfer & Network | ⏳ Pending | — | SFTP, port forwarding |
| Phase 6: Network Hardening | ⏳ Pending | — | Known hosts UI, auto-reconnect, Mosh, broadcast mode |
| Phase 7: Full-DB Encryption | ⏳ Pending | — | SQLCipher migration (defense in depth) |
| Phase 8: Biometric Unlock | ⏳ Pending | — | Touch ID, Face ID, Windows Hello, Android Fingerprint |
| Phase 9: Multi-Device Sync (E2E) | ⏳ Pending | — | iCloud, GDrive, Dropbox, S3, WebDAV adapters |
| Phase 10: Mobile Apps | ⏳ Pending | — | Android first, iOS next |
| Phase 11: Team Vault | ⏳ Pending | — | Shared host configs, key rotation |
| Phase 12: Plugin System | ⏳ Pending | — | Wasmtime sandbox, capability permissions |
| Phase 13: Audit Log | ⏳ Pending | — | Opt-in per host, encrypted, exportable |
| Phase 14: Polish & Distribution | ⏳ Pending | — | Code signing, auto-updater, a11y, release |

---

## 1. Development Methodology

- **Approach:** Scope-driven, iterative; each phase has explicit acceptance criteria and ships when met
- **Version Control:** Git with feature branch workflow
- **Code Reviews:** Required for all merges to main
- **Testing:** Unit tests for critical paths (crypto, vault, db), integration tests for SSH (Docker test container), E2E for full flows
- **No fixed timeline** — quality gates over deadlines

### 1.1 Team Structure
- **Primary Developer:** Matt (Full-stack)
- **AI Assistant:** OpenCode / Claude (code generation, debugging, documentation review)

### 1.2 Development Environment
- **OS:** Windows 11 (primary), macOS/Linux (cross-platform testing)
- **IDE:** VS Code / Cursor / OpenCode
- **Terminal:** PowerShell / Windows Terminal / Git Bash
- **Package Manager:** npm (frontend), Cargo (backend)
- **Rust toolchain:** MSVC stable (pinned via `rust-toolchain.toml`)

### Phase 1 Deliverables (Done)

- ✅ Tauri v2 scaffold (`src-tauri/Cargo.toml`, `tauri.conf.json`, `capabilities/default.json`, `build.rs`, `main.rs`, `lib.rs`)
- ✅ Frontend scaffold (`package.json`, `vite.config.ts`, `tsconfig.json` strict, Tailwind 3 with custom dark palette, PostCSS)
- ✅ SQLite database with full schema migrations (`src-tauri/src/db/`)
  - Tables: `groups`, `credentials`, `hosts`, `snippets`, `port_forwards`, `settings`, `_migrations`
  - WAL mode, foreign keys, parameterized queries
- ✅ Tauri commands: `get_hosts`, `create_host`, `update_host`, `delete_host`, `get_settings`, `set_setting`, `app_version`
- ✅ AppState with `parking_lot::Mutex<Connection>`
- ✅ AppError + serde-serializable error type
- ✅ Layout components: `AppLayout`, `TitleBar` (custom, drag region), `Sidebar`, `TabBar`, `StatusBar`, `ContentArea`
- ✅ Zustand stores: `tab-store`, `ui-store`, `host-store`
- ✅ Typed Tauri invoke wrapper (`src/lib/tauri.ts`)
- ✅ i18n strings module (`src/i18n/en.ts`)
- ✅ ESLint + Prettier + tsconfig strict (all checks pass)
- ✅ MIT LICENSE
- ✅ Verified: `npm run typecheck` ✓, `npm run lint` ✓, `npm run build` ✓ (197 KB / 62 KB gzipped), `cargo build` ✓ (MSVC toolchain)

### Phase 1 Decisions Made During Implementation

- **Package manager**: npm (not Bun) — Bun on user's system was POSIX-only binary. npm gives broader Windows compatibility.
- **Rust toolchain**: MSVC (`stable-x86_64-pc-windows-msvc`) pinned via `rust-toolchain.toml`. Initial mingw-gnu attempt failed with "export ordinal too large" — known mingw + Tauri issue on Windows. MSVC is the recommended toolchain for Tauri on Windows.
- **Window decorations**: disabled (`decorations: false`) for custom title bar.
- **Database location**: OS-standard app data dir (`%APPDATA%\com.shellmate.app\` on Windows) via Tauri `app_data_dir()`.
- **Frontend bundle baseline**: 62 KB gzipped — well within 500 KB budget.

### Phase 2 Deliverables (Done)

**Crypto primitives** (`src-tauri/src/crypto/`):
- ✅ `kdf.rs` — Argon2id key derivation (64 MiB / 3 iter / 4 parallel / 32-byte output) per OWASP guidance
- ✅ `aes.rs` — AES-256-GCM encrypt/decrypt with random 12-byte nonce per encryption
- ✅ `secure_buffer.rs` — `SecureBuffer` wrapper that zeroizes on drop, intentionally `!Clone` and `!Debug`
- ✅ Unit tests: roundtrip, wrong-key fail, tampered-ciphertext fail, deterministic derivation, salt sensitivity

**Vault** (`src-tauri/src/vault/`):
- ✅ Vault state machine (uninitialized → setup → unlocked ↔ locked)
- ✅ Master password verifier blob (encrypted constant compared via `subtle::ct_eq`)
- ✅ Password policy: 12-128 chars, length-first per NIST SP 800-63B
- ✅ Idle auto-lock check (`vault_check_idle` Tauri command, default 15 min)
- ✅ Manual lock zeroizes derived key

**Vault commands** (`src-tauri/src/commands/vault.rs`):
- ✅ `vault_status`, `vault_setup`, `vault_unlock`, `vault_lock`, `vault_check_idle`, `vault_record_activity`

**Credentials** (`src-tauri/src/commands/credential.rs`):
- ✅ `save_credential` — encrypts plaintext via vault key, stores ciphertext + nonce in SQLite
- ✅ `delete_credential`
- ✅ Internal `load_credential_plaintext` (Rust-only, never exposed to frontend) used by SSH connect

**SSH** (`src-tauri/src/ssh/`):
- ✅ `handler.rs` — minimal russh client handler (TOFU host key acceptance for now, known_hosts UI in Phase 6)
- ✅ `session.rs` — `SessionManager` with `Arc<RwLock<HashMap>>`, one async task per session
  - **Strategy**: 1 SSH connection per tab (per docs/04-backend-plan §9)
  - PTY request, shell channel, keepalive (60s interval, max 3 retries)
  - Bidirectional I/O loop: keystrokes via mpsc channel → russh, server data → Tauri events
  - Per-session events: `ssh:output:{id}`, `ssh:status:{id}`, `ssh:error:{id}`
  - MAX_SESSIONS = 50, SOFT_SESSION_LIMIT = 20
- ✅ Auth methods: password and private key (with optional passphrase)

**SSH commands** (`src-tauri/src/commands/ssh.rs`):
- ✅ `ssh_connect` — connect by host_id (loads + decrypts credential via vault)
- ✅ `ssh_quick_connect` — one-off connection without saving credential (for testing & demo)
- ✅ `ssh_send`, `ssh_resize`, `ssh_disconnect`

**Frontend**:
- ✅ `stores/vault-store.ts` — vault state with refresh/setup/unlock/lock/recordActivity
- ✅ `stores/ssh-store.ts` — tab id ↔ SSH session id mapping
- ✅ `components/vault/VaultGate.tsx` — gates the app behind vault unlock
- ✅ `components/vault/VaultSetup.tsx` — first-run setup with mandatory recovery warning + acknowledge checkbox (per 07-security §4.1.2)
- ✅ `components/vault/VaultUnlock.tsx` — unlock form
- ✅ `components/terminal/Terminal.tsx` — xterm.js wrapper with FitAddon, WebLinksAddon, SSH event subscription, ResizeObserver
- ✅ `components/connect/QuickConnect.tsx` — form for testing SSH connections (clears sensitive fields after submit)
- ✅ `ContentArea` renders all bound terminals with visibility toggling so xterm state survives tab switches
- ✅ `TabBar` cleanup: disconnects backend session and unbinds on tab close
- ✅ `StatusBar` lock button (with disabled state when already locked)
- ✅ Typed Tauri wrapper extended for vault, credentials, SSH commands

**Verified**:
- ✅ `npm run typecheck` exit 0
- ✅ `npm run lint` exit 0
- ✅ `npm run format:check` clean
- ✅ `npm run build` — 509 KB / 140 KB gzipped (still within 500 KB gzipped budget)
- ✅ `cargo build` — MSVC, 8 unused-API warnings (all forward-compat, will be used Phase 3+)

### Phase 2 Decisions Made During Implementation

- **russh version**: 0.45 (not 0.50+). Older API: `authenticate_*` returns `bool`, `client::Handler::check_server_key` takes `key::PublicKey` (not `ssh_key::PublicKey`), `decode_secret_key` returns `KeyPair` directly used as `Arc<KeyPair>` for `authenticate_publickey`. 0.50+ has breaking changes that require additional adapter work — defer upgrade until needed.
- **Host key verification**: TOFU-accepting handler for now. Known_hosts table + verification UI in Phase 6 (per 07-security §6.1).
- **Verifier scheme**: Use a fixed plaintext (`b"shellmate.vault.v1"`) encrypted with derived key. Decryption + constant-time compare proves password without storing key hash separately. AES-GCM auth tag already provides integrity.
- **No password recovery**: hardcoded into UX per 07-security §4.1.2. Setup form blocks submit until user explicitly checks the acknowledgement.
- **xterm tab persistence**: `ContentArea` keeps all terminals mounted with `visibility: hidden` to preserve state across tab switches. Avoids xterm reinit cost and scrollback loss.
- **Multi-tab one-connection-per-tab**: implemented per docs/04-backend-plan §9. Each tab opens a fresh `client::connect`. SOFT_SESSION_LIMIT and MAX_SESSIONS constants ready to wire UI warnings in Phase 5.
- **PTY**: `xterm-256color`, 80x24 initial. Frontend `ResizeObserver` triggers `ssh_resize` on window or sidebar changes.
- **Disk space discovered during cargo build**: 9 GB target dir. After `cargo clean`, freed ~9 GB. Note for Phase 3+: target dir grows quickly with russh + tokio dependency tree.

---

## 2. Phase 1: Project Setup ✅

**Status:** Complete (2026-06-09). See §0 Progress Tracker for deliverables and decisions.

---

## 3. Phase 2: Core SSH ✅

**Status:** Complete (2026-06-10). See §0 Progress Tracker for deliverables and decisions.

---

## 4. Phase 3: Host Management & Persistence ✅

**Status:** Complete (2026-06-10)

### Acceptance Criteria
- [x] Host CRUD UI: form (add, edit), list, delete with confirmation
- [x] Validation matches backend rules (hostname, port range, auth_type, username)
- [x] Group CRUD: create, rename, delete (orphan handling), color, nesting
- [x] Drag-and-drop: reorder hosts within group, move between groups
- [ ] Tag input with autocomplete from existing tags (basic comma-separated input shipped; autocomplete deferred to Phase 4)
- [x] Notes field (markdown preview deferred to Phase 4)
- [x] Host search: hostname, label, tag, group name, notes (client-side filter for performance)
- [x] Save credential via vault, connect from sidebar (uses `ssh_connect`)
- [x] Empty states for no hosts, no groups, no search results

### Phase 3 Deliverables (Done)

**Backend** (`src-tauri/src/commands/`):
- ✅ `group.rs` — `get_groups`, `create_group`, `update_group`, `delete_group`, `move_host_to_group`
  - Validation: name required, hex color format check, prevent self-parent cycle
  - Delete cascades: hosts in deleted group become ungrouped, sub-groups become detached
- ✅ `host.rs` extended with `search_hosts` — multi-field LIKE query (label, hostname, username, group name, tags, notes)
- ✅ Wired all new commands into `lib.rs`

**Frontend**:
- ✅ `lib/tauri.ts` extended with `groups.list/create/update/delete` and `hosts.search/moveToGroup`
- ✅ `types/host.ts` added `GroupInput` interface
- ✅ `stores/host-store.ts` rewritten with full state: `hosts`, `groups`, `searchQuery`, `expandedGroups`, plus actions for CRUD on both entities, search, group expansion, and host-to-group moves
- ✅ UI primitives:
  - `components/ui/Button.tsx` (4 variants × 2 sizes)
  - `components/ui/Form.tsx` (`Input`, `Textarea`, `Select`, `Field` with error/hint)
  - `components/ui/Modal.tsx` (focus trap, Esc-to-close, click-outside, accessible)
  - `components/ui/ConfirmDialog.tsx`
- ✅ Host management:
  - `components/hosts/HostForm.tsx` — add/edit modal with validation, supports password and SSH key auth, tag parsing, notes
  - `components/hosts/HostItem.tsx` — sidebar item with drag-and-drop, right-click context menu (connect/edit/delete), double-click to connect, group color dot, hostname tooltip
  - `components/hosts/HostList.tsx` — grouped sections, expand/collapse, drag-over visual feedback, empty + no-results states
  - `components/hosts/GroupForm.tsx` — create/edit with preset color swatches + custom hex input
- ✅ `Sidebar` rewritten:
  - Loads hosts + groups when vault unlocks
  - Connected search input to host store
  - Renders real `HostList` (replaces placeholder groups)
  - Add Host + New Group action buttons

### Phase 3 Decisions Made During Implementation

- **Search strategy**: client-side filter using already-loaded hosts (since dataset is small, ms response). Backend `search_hosts` available for future server-side filter (e.g., when audit log query joins).
- **Drag-and-drop**: native HTML5 drag-and-drop with `application/x-shellmate-host` MIME. Group section is drop target; ungrouped section is also a target. No external lib needed.
- **Group sort**: sort by `sort_order` first, then alphabetical. UI for reordering groups deferred (drag groups themselves) — Phase 4 candidate.
- **Credential lifecycle**: when editing a host, blank credential field means "keep existing"; non-blank creates a new credential row. Old credential row is NOT auto-deleted on host edit (could be referenced elsewhere later) — only on host delete.
- **Tags**: comma-separated input. Autocomplete with existing tags deferred to Phase 4.
- **Markdown notes**: plain textarea for now. Rich preview deferred to Phase 4.
- **Modal lib**: lightweight in-house Modal (~50 lines) to avoid pulling shadcn/Dialog + @radix-ui/react-dialog before we have full design system needs. Will swap to shadcn/ui Dialog in Phase 4 when more dialog patterns appear.
- **UI primitives prefix**: `components/ui/` (matches shadcn convention). Avoid `components/common/` etc. for consistency.
- **Group nesting**: schema supports `parent_id` (Phase 1 schema), backend enforces self-parent rejection. UI for tree-display nested groups not yet implemented — flat group list shipped. Tree UI deferred to Phase 4 if user feedback demands it.
- **Connect flow**: click connect (or Enter / double-click) creates a frontend tab and calls `ssh_connect(host_id)`. Backend pulls credential via vault, frontend `bind`s tab id ↔ session id. Existing Phase 2 Terminal subscribes to events automatically.

### Verified
- ✅ `npm run typecheck` exit 0
- ✅ `npm run lint` exit 0
- ✅ `npm run format:check` clean
- ✅ `npm run build` — 533 KB / 147 KB gzipped (within 500 KB gzipped budget)
- ✅ `cargo build` — incremental 0.73s, 8 forward-compat warnings (unchanged from Phase 2)

---

## 5. Phase 4: Productivity & Settings

### Acceptance Criteria
- [ ] Snippet CRUD with template variables (`{{username}}`, `{{host}}`, custom)
- [ ] Snippet panel (Ctrl+K), search, execute to active terminal
- [ ] Settings dialog: General, Terminal, Vault, Shortcuts, Theme
- [ ] Custom theme editor: terminal palette + UI tokens, preview, export/import theme JSON
- [ ] Keyboard shortcut customization with conflict detection
- [ ] Auto-lock UX: frontend polls `vault_check_idle`, dispatches lock when fired
- [ ] Master password change: re-derives key, re-encrypts all credentials, atomic
- [ ] Settings persist to SQLite `settings` table

### Out of Scope
- SFTP, port forwarding (Phase 5)
- Multi-device sync of settings (Phase 9)

---

## 6. Phase 5: File Transfer & Network

### Acceptance Criteria
- [ ] SFTP browser opens as panel within active session
- [ ] Directory listing: name, size, permissions, modified date, file type icon
- [ ] Navigation: up, into directory, breadcrumb, address bar
- [ ] Operations: upload (drag-drop + picker), download, rename, delete, mkdir
- [ ] Progress indicator for transfers > 1 MB
- [ ] Multiple SFTP windows per session
- [ ] SFTP runs as separate channel on parent SSH connection
- [ ] Port forwarding: local (-L) and remote (-R) rules per host
- [ ] Toggle rule on/off without disconnecting
- [ ] Conflict detection: clear error if local port already bound

### Out of Scope
- SFTP search (post-1.0)
- Dynamic forwarding (-D) (post-1.0)

---

## 7. Phase 6: Network Hardening

### Acceptance Criteria
- [ ] Known hosts table populated on first connect (TOFU)
- [ ] Verification UI: show fingerprint, ask user to trust
- [ ] On key mismatch: warning dialog with old vs new fingerprint, options (trust new, abort, view details)
- [ ] Auto-reconnect: exponential backoff (1s, 2s, 5s, 10s, 30s, 60s max)
- [ ] User-cancellable reconnect with status visible in tab
- [ ] **Mosh client**: spawn mosh-server via SSH, then UDP transport
- [ ] Mosh tab shows roaming/dropped state distinctly
- [ ] **Broadcast mode**: select multiple tabs, single input field broadcasts keystrokes
- [ ] Visual indicator on broadcasted tabs

### Out of Scope
- Cloud provider integration (post-1.0)

---

## 8. Phase 7: Full-DB Encryption (SQLCipher)

### Acceptance Criteria
- [ ] Migration tool: detect existing plaintext SQLite DB, prompt user, re-create with SQLCipher
- [ ] Master password derivation produces both vault key (Argon2id) and DB key (separate output via HKDF)
- [ ] All existing per-credential AES-GCM encryption stays in place (defense in depth)
- [ ] Read/write performance regression < 15% on benchmark
- [ ] Migration is atomic: succeeds fully or rolls back
- [ ] Backup of pre-migration DB is created before migration

### Out of Scope
- Plaintext DB support (drop after migration)

---

## 9. Phase 8: Biometric Unlock

### Acceptance Criteria
- [ ] Touch ID (macOS), Windows Hello, Android BiometricPrompt, iOS Face/Touch ID
- [ ] Vault key wrapped with biometric-protected secure enclave key
- [ ] Fallback to master password if biometric fails or is disabled
- [ ] User can enable/disable biometric per device in settings
- [ ] Biometric state survives app restart
- [ ] Failed biometric attempts do NOT count toward master password lockout

### Out of Scope
- Hardware key auth like YubiKey (post-1.0)

---

## 10. Phase 9: Multi-Device Sync (E2E)

### Acceptance Criteria
- [ ] Sync engine: encrypt-then-upload, manifest tracking, version vector clocks
- [ ] Backend adapters: iCloud (macOS/iOS), GDrive, Dropbox, S3, WebDAV, self-hosted endpoint
- [ ] Selective sync: per host, per snippet, per group
- [ ] Conflict resolution: last-write-wins by default, manual merge UI for marked conflicts
- [ ] Sync log + diagnostic panel (last sync time, errors, data transferred)
- [ ] Pause/disable any time
- [ ] Verification: cloud provider cannot read sync payload (manual test with `aws s3 cp` etc.)

### Security Acceptance
- [ ] All payloads encrypted with vault-derived key before upload
- [ ] No metadata leakage in object names, paths, or headers (use opaque IDs)

### Out of Scope
- Real-time sync (sync triggered on change with debounce, or manual)
- Multi-user merge (covered by Team Vault, Phase 11)

---

## 11. Phase 10: Mobile Apps (Android & iOS)

### Acceptance Criteria
- [ ] Tauri v2 mobile target builds successfully for Android and iOS
- [ ] Adaptive UI: bottom-sheet navigation, full-screen panels, swipe between tabs
- [ ] **Extended key bar**: Esc, Tab, Ctrl, Alt, ↑↓←→, |, ~, -, /, configurable
- [ ] Touch-friendly host list, tab switcher, SFTP modal
- [ ] Pinch-to-zoom on terminal font size
- [ ] Auto-rotate (portrait + landscape)
- [ ] Background reconnect with notification on disconnect
- [ ] Biometric unlock works (Phase 8 prerequisite)
- [ ] Performance: cold start < 3s, 60fps scroll

### Out of Scope
- Tablet-specific UI optimization (use phone UI scaled up for v1.0)

---

## 12. Phase 11: Team Vault

### Acceptance Criteria
- [ ] Team creation: generate team key pair (encrypted with team master password)
- [ ] Member management: add member by public key, revoke, key rotation
- [ ] Per-host share: select hosts to share with team, set permissions (read-only / edit)
- [ ] Encrypted host config wrapped with team key
- [ ] Conflict resolution: same as personal sync, last-write-wins + merge UI
- [ ] Audit trail of share/revoke events (only when audit log Phase 13 also enabled)

### Out of Scope
- Roles & RBAC beyond read/edit (post-1.0)
- SSO integration (post-1.0)

---

## 13. Phase 12: Plugin System

### Acceptance Criteria
- [ ] Wasmtime runtime integrated, sandboxed
- [ ] Plugin API hooks: `pre_connect`, `post_connect`, `terminal_data_in`, `terminal_data_out`, `register_panel`
- [ ] Capability-based permissions: `network`, `filesystem`, `secrets` — all opt-in per plugin via manifest
- [ ] Plugin manifest format with signature
- [ ] Plugin distribution: load from local file (drag-drop or file picker)
- [ ] Plugin permissions UI: review on install, revoke later
- [ ] Sample plugins shipped: theme installer, prompt customizer, log viewer
- [ ] Plugin crashes do NOT crash the host app

### Out of Scope
- Public plugin registry (post-1.0)
- WASI advanced features beyond what plugin API needs

---

## 14. Phase 13: Audit Log

### Acceptance Criteria
- [ ] Audit event capture: session start/end, SFTP transfers, command history (opt-in per host)
- [ ] Encrypted audit log storage (own vault key)
- [ ] Viewer UI: filter by host, date range, event type, search
- [ ] Export: signed JSONL with timestamps and event chain hash
- [ ] Privacy: redaction rules for known patterns (passwords in command, etc.)
- [ ] Retention policy: configurable (30/60/90/365 days, never)

### Out of Scope
- Real-time alerts (post-1.0)
- SIEM integration (post-1.0)

---

## 15. Phase 14: Polish & Distribution

### Acceptance Criteria
- [ ] Onboarding flow: first-launch tutorial, vault setup walkthrough, sample data offer
- [ ] Error handling: toast notifications, reconnect UI for disconnected sessions
- [ ] Encrypted host export/import (offline backup option)
- [ ] Performance audit: bundle size, startup, memory all within targets
- [ ] **Full a11y pass**: axe-core CI gate, manual NVDA + VoiceOver smoke test
- [ ] Cross-platform testing: Windows, macOS, Linux, Android, iOS
- [ ] **Code signing**: Windows Authenticode, macOS notarization, Linux GPG
- [ ] **Tauri auto-updater**: signed releases, opt-in beta channel
- [ ] App packaging: Windows .msi, macOS .dmg (Intel + ARM), Linux .AppImage + .deb, Android .apk + .aab, iOS via TestFlight
- [ ] User documentation: install guide, getting started, features, troubleshooting per platform
- [ ] Release v1.0.0

---

## 8. Technical Decisions

### 8.1 State Management
**Decision:** Zustand
**Reasoning:** Lightweight, simple API, good TypeScript support, minimal boilerplate.

### 8.2 Styling
**Decision:** Tailwind CSS + shadcn/ui components
**Reasoning:** Rapid development, consistent design, accessible components.

### 8.3 Terminal Emulator
**Decision:** xterm.js
**Reasoning:** Industry standard, feature-rich, good performance, active maintenance.

### 8.4 SSH Implementation
**Decision:** russh (Rust)
**Reasoning:** Native performance, memory safety, direct Tauri integration.

### 8.5 Database
**Decision:** SQLite via rusqlite
**Reasoning:** Single file database, no server required, reliable, well-tested.

### 8.6 Encryption
**Decision:** AES-256-GCM + Argon2id
**Reasoning:** Strong encryption, memory-hard key derivation, industry standard.

---

## 9. Risk Mitigation

### 9.1 Technical Risks

| Risk | Impact | Mitigation |
|------|--------|------------|
| SSH connection stability | High | Implement keepalive, auto-reconnect, thorough testing |
| Cross-platform compatibility | Medium | Test on all platforms early, use conditional compilation |
| Performance with many tabs | Medium | Implement tab lazy loading, optimize memory usage |
| Encryption key management | High | Follow security best practices, thorough testing |

### 9.2 Schedule Risks

| Risk | Impact | Mitigation |
|------|--------|------------|
| Scope creep | High | Acceptance-criteria-driven phases; out-of-scope items move to post-1.0 backlog explicitly |
| Technical blockers | Medium | Research key libraries early, have fallback options |
| Testing time | Medium | Write tests alongside code, not after |

---

## 10. Definition of Done

### 10.1 Feature Done
- [ ] Code complete and working
- [ ] Unit tests written and passing
- [ ] Code reviewed
- [ ] Documentation updated
- [ ] No known bugs

### 10.2 Sprint Done
- [ ] All planned features complete
- [ ] All tests passing
- [ ] Build successful on all platforms
- [ ] No critical bugs
- [ ] Demo ready

### 10.3 Release Done
- [ ] All v1.0 phase acceptance criteria met
- [ ] Cross-platform testing passed (Windows, macOS, Linux, Android, iOS)
- [ ] Code-signed and notarized installers
- [ ] Auto-updater verified
- [ ] User documentation complete
- [ ] Release notes written
- [ ] Security review pass

---

## 11. Communication

### 11.1 Daily
- Progress updates in Telegram
- Blockers identified immediately

### 11.2 Weekly
- Sprint review
- Demo of completed features
- Planning for next week

---

*This development plan defines a scope-driven path to ShellMate v1.0 production release. Phases ship when acceptance criteria are met — no fixed deadlines.*
