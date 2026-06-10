# Project Scope
## ShellMate — SSH Client (Production v1.0)

**Version:** 2.0
**Last Updated:** 2026-06-10

---

## 1. Executive Summary

ShellMate is a self-hosted, local-first SSH client built as a **full v1.0 production release** — not an MVP. It targets developers, DevOps engineers, system administrators, and small teams who need a modern multi-device SSH workflow without paying for a SaaS or trusting a third-party cloud.

The product runs on Windows, macOS, Linux, Android, and iOS (Tauri v2 mobile). Optional end-to-end-encrypted sync uses the user's own cloud (iCloud, GDrive, Dropbox, S3, WebDAV) or self-hosted endpoint — there is no ShellMate server in the loop.

---

## 2. Product Goals

### 2.1 Primary Goals
1. **Multi-SSH Connection** — Many concurrent SSH sessions in tabs with broadcast capability
2. **Multi-Device, Multi-Platform** — Desktop (Win/macOS/Linux) + Mobile (Android/iOS) with consistent UX
3. **End-to-End Encrypted Sync** — Optional sync via user's own cloud, no ShellMate servers
4. **Local-First & Privacy by Default** — All state on device, telemetry zero, defense-in-depth encryption (AES-256-GCM + SQLCipher)
5. **Extensible** — Plugin system (Wasmtime sandbox), custom themes, team vault

### 2.2 Non-Functional Targets

| Metric | Target |
|--------|--------|
| Cold start time | < 2s desktop, < 3s mobile |
| Memory usage (idle, desktop) | < 80 MB |
| Memory usage (5 tabs, desktop) | < 150 MB |
| SSH connection time | < 1s after vault unlocked |
| Desktop binary size | < 30 MB installer |
| Mobile binary size | < 25 MB |
| Frontend bundle | < 500 KB gzipped |
| WCAG | 2.1 AA on UI chrome |

---

## 3. In Scope (v1.0)

ShellMate v1.0 ships as a complete production release. All items below are required before v1.0 GA.

### 3.1 Connection & Terminal
- Multi-tab SSH session, no hard limit (soft warning at 20)
- 1 SSH connection per tab (isolation strategy, see 04-backend §9)
- Password & SSH key auth (with passphrase support)
- xterm.js terminal: ANSI colors, resize, copy/paste, search, web links
- **Broadcast mode** — send command to multiple selected sessions
- SSH keepalive + auto-reconnect with exponential backoff
- Known hosts management with TOFU + warning on key mismatch
- **Mosh support** — UDP SSP fallback for unreliable networks (mobile-critical)

### 3.2 Host & Organization
- Host CRUD with validation
- Host groups with nesting + drag-and-drop reorganization
- Tags, notes, free-text search
- Encrypted host export/import (offline backup)

### 3.3 Vault & Security
- Argon2id key derivation (memory-hard, OWASP params)
- AES-256-GCM per-credential encryption
- **SQLCipher full-DB encryption** (defense in depth, protects all metadata)
- Length-first master password policy (12-128 char per NIST SP 800-63B)
- No-recovery rule + onboarding warning + acknowledgement gate
- Auto-lock after idle (configurable, default 15 min)
- Manual lock (Ctrl+L)
- Master password change with re-encryption of vault
- **Biometric unlock**: Touch ID, Face ID, Windows Hello, Android BiometricPrompt
- Memory zeroize for all secrets (Rust `zeroize`)

### 3.4 Productivity
- Snippets with template variables, search, execute to active terminal
- Settings dialog: theme, font, shortcuts, keepalive, scrollback, auto-lock
- **Custom themes** — user-defined color schemes (terminal + UI tokens)
- Configurable keyboard shortcuts

### 3.5 File Transfer
- SFTP file browser (browse, upload, download, rename, delete, mkdir)
- Drag-and-drop upload
- Progress indicator
- Multiple SFTP windows per session
- SFTP runs as separate channel on parent SSH connection

### 3.6 Network
- Port forwarding (Local `-L` and Remote `-R`)
- Toggle rules without disconnecting session
- Port conflict detection

### 3.7 Multi-Device
- Desktop: Windows 10+, macOS 12+, Linux (Ubuntu 20.04+ and equivalents)
- Mobile: Android 10+, iOS 15+ via Tauri v2 mobile target
- Mobile UI: extended key bar (Esc, Tab, Ctrl, Alt, arrows, pipe, tilde, slash), bottom-sheet navigation, full-screen SFTP modal, touch-friendly tab switcher

### 3.8 Multi-Device Sync
- **Optional**: app fully functional without sync
- User chooses backend: iCloud, GDrive, Dropbox, S3, WebDAV, or self-hosted endpoint
- Encryption: payload encrypted on device before upload (AES-256-GCM with vault-derived key)
- Conflict resolution: last-write-wins with timestamp + manual merge UI for complex conflicts
- Selective sync: choose which hosts/snippets to sync per device
- Sync log + diagnostic panel
- Pause/disable any time

### 3.9 Team & Sharing
- **Team vault** — share host configs encrypted with team key
- Member management: add via public key, revoke, key rotation
- Per-host permissions (read-only / edit)
- Conflict resolution for shared host changes

### 3.10 Plugin System
- Wasmtime runtime (WASM, sandboxed)
- Plugin API: hooks (pre/post connect, terminal data filter), custom UI panels
- Capability-based permissions: network, filesystem, secrets — all opt-in per plugin
- Plugin manifest with signature
- Distribution: load from file (no public registry in v1.0)

### 3.11 Audit & Observability
- **Audit log** — opt-in per host: session start/end, file transfers, command history
- Encrypted audit log storage
- Viewer UI with filter, search, export (signed JSONL)
- Privacy: redaction rules for secrets in command history

### 3.12 Distribution & Updates
- Code signing: Windows Authenticode, macOS notarization, Linux GPG-signed AppImage
- **Auto-updater** via Tauri v2 updater with signed releases
- Multi-arch builds: Windows x64, macOS Intel + Apple Silicon, Linux x64 + arm64, Android, iOS

### 3.13 Accessibility
- WCAG 2.1 AA on UI chrome (terminal content exempt)
- Full keyboard navigation, focus traps, ARIA labels
- Screen reader smoke test pass (NVDA on Windows, VoiceOver on macOS)
- `prefers-reduced-motion` respected
- High contrast verification

---

## 4. Out of Scope (v1.0)

- ❌ Cloud-hosted ShellMate service (always self-hosted / user's own cloud)
- ❌ Browser-based version
- ❌ Serial port / Telnet (security & modernity reasons)
- ❌ Container management (Docker, K8s) — pakai dedicated tool
- ❌ Built-in tmux replacement (use real tmux on the server)
- ❌ Telemetry / analytics
- ❌ Subscription tiers / paywalled features

## 5. Future Scope (post-1.0)

- Hardware key auth (FIDO2 / YubiKey)
- SSH agent forwarding (with explicit per-host opt-in)
- Cloud provider integration (AWS Session Manager, GCP IAP, Azure Bastion)
- Encrypted notes / runbooks per host
- Workflow automation (chain snippets across hosts)
- Public plugin registry

---

## 5. User Stories

### 5.1 Host Management
```
As a developer,
I want to add and organize my SSH hosts,
So that I can quickly connect to my servers.

Acceptance Criteria:
- I can add a host with hostname, port, username, and auth method
- I can organize hosts into groups (Production, Staging, Dev)
- I can search hosts by name or hostname
- I can edit and delete hosts
- All changes save automatically
```

### 5.2 Multi-Terminal
```
As a DevOps engineer,
I want to connect to multiple servers simultaneously,
So that I can manage my infrastructure efficiently.

Acceptance Criteria:
- I can open multiple terminal tabs
- Each tab has its own SSH session
- I can switch between tabs with Ctrl+Tab
- I can see connection status per tab
- I can close tabs with confirmation
```

### 5.3 Secure Credentials
```
As a security researcher,
I want my credentials stored securely,
So that my servers remain protected.

Acceptance Criteria:
- Credentials are encrypted at rest
- I set a master password on first use
- App locks after idle timeout
- I can manually lock with Ctrl+L
- Credentials never appear in logs
```

### 5.4 Quick Commands
```
As a system administrator,
I want to save frequently used commands,
So that I can execute them quickly.

Acceptance Criteria:
- I can save commands as snippets
- I can execute snippets to active terminal
- I can search snippets by name
- I can organize snippets with tags
```

---

## 6. Technical Constraints

### 6.1 Technology Stack
| Layer | Technology | Constraint |
|-------|------------|------------|
| Framework | Tauri v2 | Desktop + mobile target |
| Frontend | React 18 + Vite + TypeScript | strict mode |
| Styling | Tailwind CSS 3 + shadcn/ui | accessible by default |
| Terminal | xterm.js | with FitAddon, SearchAddon, WebLinksAddon |
| SSH | russh (Rust) | memory-safe |
| Mosh | Rust port (Phase 6) | UDP SSP |
| Database | SQLite + SQLCipher (Phase 7) | full-DB encryption |
| Encryption | AES-256-GCM + Argon2id | per-credential layer + DB layer |
| Plugin Runtime | Wasmtime (Phase 12) | sandboxed WASM |
| State | Zustand | lightweight, minimal boilerplate |
| Package Manager | npm + Cargo | cross-platform |

### 6.2 Performance Constraints
- Cold start: < 2s desktop, < 3s mobile
- Memory idle: < 80 MB desktop
- Memory 5 tabs: < 150 MB desktop
- SSH overhead: < 5ms vs native client
- Desktop installer: < 30 MB
- Mobile binary: < 25 MB
- Frontend bundle: < 500 KB gzipped (CI gate)

### 6.3 Security Constraints
- All credentials encrypted at rest (per-credential AES-GCM + SQLCipher full-DB)
- No plaintext logging
- No telemetry / analytics
- No cloud dependency (sync uses user's own cloud)
- Zeroize all secrets from memory after use
- WASM plugin sandbox with explicit capability grants

---

## 7. Assumptions

### 7.1 User Assumptions
- Users have basic SSH knowledge
- Users understand terminal commands
- Users have SSH servers to connect to
- Users prefer local-first applications
- Users with multi-device needs already have a cloud account (iCloud/GDrive/Dropbox/S3) or self-hosted endpoint

### 7.2 Technical Assumptions
- Target servers have SSH (and optionally Mosh) servers running
- Network connectivity available for SSH (reasonable latency for SSH; Mosh tolerates lossy networks)
- Sufficient disk space for SQLite + SQLCipher database
- OS supports WebView2 (Windows), WKWebView (macOS/iOS), or system WebView (Linux/Android)
- Mobile devices support biometric APIs (else fallback to master password)

---

## 8. Dependencies

### 8.1 External Dependencies
| Dependency | Version | Purpose |
|------------|---------|---------|
| Tauri | v2.x | App framework (desktop + mobile) |
| russh | 0.45 (pinned) | SSH implementation |
| xterm.js | 5.x | Terminal emulator |
| rusqlite | 0.32 | SQLite bindings |
| SQLCipher | latest stable | Full-DB encryption (Phase 7) |
| Wasmtime | latest | Plugin sandbox (Phase 12) |
| React | 18.x | UI framework |
| Vite | 6.x | Build tool |
| Tailwind CSS | 3.x | Styling |
| shadcn/ui | latest | Accessible component primitives |

### 8.2 Development Dependencies
| Dependency | Purpose |
|------------|---------|
| Node.js | Frontend tooling |
| npm | Package manager |
| Rust | Backend development |
| Cargo | Rust package manager |
| Git | Version control |
| Apple Developer cert (Phase 14) | macOS notarization + iOS distribution ($99/yr) |
| Windows Code Signing cert (Phase 14) | Authenticode signing |

---

## 9. Deliverables

### 9.1 v1.0 Production Release
1. **Application Binaries (signed)**
   - Windows MSI installer (Authenticode-signed)
   - macOS DMG (notarized) — Intel + Apple Silicon
   - Linux AppImage (GPG-signed) + .deb
   - Android APK + AAB (Play Store-ready)
   - iOS via TestFlight, then App Store

2. **Source Code**
   - Complete Tauri v2 project
   - All source files documented
   - Unit tests for critical paths (crypto, vault, db)
   - Integration tests for SSH (Docker test container)

3. **Documentation**
   - README.md (overview, install, getting started)
   - User guide (per platform)
   - Developer documentation (architecture, contributing)
   - Security audit notes
   - Plugin SDK documentation

4. **Distribution Infrastructure**
   - Tauri auto-updater endpoint with signed manifests
   - GitHub Releases with checksums

---

## 10. Phasing & Timeline

ShellMate v1.0 is delivered **scope-driven** — no fixed deadlines. Each phase ships when acceptance criteria are met.

| Phase | Status | Area |
|-------|--------|------|
| 1 | ✅ Complete (2026-06-09) | Project Setup |
| 2 | ✅ Complete (2026-06-10) | Core SSH + Vault + Terminal |
| 3 | ⏳ Pending | Host Management & Persistence |
| 4 | ⏳ Pending | Productivity & Settings (snippets, themes, shortcuts) |
| 5 | ⏳ Pending | File Transfer & Network (SFTP, port forward) |
| 6 | ⏳ Pending | Network Hardening (known hosts, Mosh, broadcast) |
| 7 | ⏳ Pending | Full-DB Encryption (SQLCipher migration) |
| 8 | ⏳ Pending | Biometric Unlock |
| 9 | ⏳ Pending | Multi-Device Sync (E2E) |
| 10 | ⏳ Pending | Mobile Apps (Android, iOS) |
| 11 | ⏳ Pending | Team Vault |
| 12 | ⏳ Pending | Plugin System (Wasmtime) |
| 13 | ⏳ Pending | Audit Log |
| 14 | ⏳ Pending | Polish & Distribution |

See `docs/01-development-plan.md` for detailed deliverables per phase.

---

## 11. Success Criteria

### 11.1 Functional
- [ ] Add/edit/delete/group/search hosts with drag-and-drop
- [ ] Connect to many SSH servers simultaneously, broadcast to multiple
- [ ] Mosh fallback for unreliable networks
- [ ] Credentials encrypted at rest (per-cred + full-DB)
- [ ] Biometric unlock works on all target platforms
- [ ] Sync works across desktop and mobile
- [ ] Snippets, settings, custom themes, configurable shortcuts
- [ ] SFTP browser, port forwarding
- [ ] Team vault with key rotation
- [ ] Plugin system runs sandboxed third-party WASM
- [ ] Audit log captures opt-in events

### 11.2 Technical
- [ ] All performance targets met (see §2.2)
- [ ] Builds and runs on Windows, macOS, Linux, Android, iOS
- [ ] No crashes or data loss in 30-day soak test
- [ ] All unit + integration tests pass in CI
- [ ] Code coverage targets met (per docs/08-devops §14.1)

### 11.3 Security
- [ ] Independent security review pass
- [ ] No critical vulnerabilities in dependency audit
- [ ] Vault recovery scenarios tested (lost device, stolen device)
- [ ] Sync E2E encryption verified (cloud provider cannot read payloads)

### 11.4 User Experience
- [ ] WCAG 2.1 AA pass for UI chrome
- [ ] Manual screen reader smoke test pass (NVDA + VoiceOver)
- [ ] Keyboard-only navigation walkthrough pass
- [ ] Onboarding flow tested with 5+ first-time users
- [ ] Mobile UX tested on Android + iOS, multiple form factors
- [ ] Error messages actionable

---

*This document defines the complete scope of ShellMate v1.0 production release. Items not listed here are post-1.0.*
