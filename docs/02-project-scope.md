# Project Scope
## ShellMate - SSH Client Desktop App

**Version:** 1.0
**Last Updated:** 2026-06-07

---

## 1. Executive Summary

ShellMate is a self-hosted, local-first SSH client desktop application designed for developers, DevOps engineers, and system administrators who need to manage multiple SSH connections efficiently. The app prioritizes security, performance, and user experience while keeping all data local to the user's device.

---

## 2. Product Goals

### 2.1 Primary Goals
1. **Multi-SSH Connection** - Connect to multiple servers simultaneously via tabs
2. **Security First** - Encrypted credentials, no cloud dependency
3. **Modern UX** - Clean, fast, keyboard-first interface
4. **Cross-Platform** - Works on Windows, macOS, and Linux

### 2.2 Success Metrics
| Metric | Target |
|--------|--------|
| Cold start time | < 2 seconds |
| Memory usage (idle) | < 50MB |
| Memory usage (5 tabs) | < 100MB |
| SSH connection time | < 1 second |
| Binary size | < 20MB |

---

## 3. In Scope (MVP - Desktop)

### 3.1 Core Features

#### 3.1.1 Host Management
- Add, edit, delete SSH hosts
- Organize hosts into groups
- Search hosts by name/hostname
- Host notes and tags
- Import/export hosts (JSON)

#### 3.1.2 SSH Terminal
- Interactive terminal via xterm.js
- Multi-tab sessions (unlimited)
- SSH keepalive support
- Terminal resize
- Copy/paste
- Terminal search
- Status indicators (Connected/Connecting/Disconnected)

#### 3.1.3 Authentication
- Password authentication
- SSH key authentication
- Passphrase-protected keys
- Secure credential storage

#### 3.1.4 Credential Vault
- AES-256-GCM encryption
- Argon2id key derivation
- Master password protection
- Auto-lock after idle
- Manual lock (Ctrl+L)

#### 3.1.5 Snippets
- Save frequently used commands
- Execute snippets to terminal
- Search snippets
- Snippet tags/categories

#### 3.1.6 Settings
- Theme (Dark/Light/System)
- Terminal font and size
- Cursor style
- Keyboard shortcuts
- Auto-lock timeout

### 3.2 Power Features

#### 3.2.1 Port Forwarding
- Local port forwarding (-L)
- Remote port forwarding (-R)
- Port forwarding management
- Conflict detection

#### 3.2.2 SFTP File Browser
- Browse remote files
- Upload files (drag-and-drop)
- Download files
- File operations (rename, delete, mkdir)
- Permission display

### 3.3 Platform Support
- Windows 10+
- macOS 12+
- Ubuntu 20.04+

---

## 4. Out of Scope (MVP)

### 4.1 Explicitly Excluded
- ❌ Mobile apps (Android/iOS)
- ❌ Cloud sync/backup
- ❌ Serial/Telnet connections
- ❌ Terminal multiplexer (tmux-like)
- ❌ Container management (Docker, K8s)
- ❌ Plugin system
- ❌ Team/sharing features
- ❌ Audit logging
- ❌ Multi-device sync
- ❌ Auto-updater
- ❌ Crash reporting
- ❌ Analytics/telemetry

### 4.2 Deferred to Post-MVP
- Mobile app with touch-optimized UI
- Extended key bar for mobile
- iCloud/GDrive/S3 sync
- Biometric unlock
- Broadcast mode (command to multiple servers)
- Export/import encrypted configs
- Custom themes
- Macros/automation

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
| Framework | Tauri v2 | Must use WebView, not Chromium |
| Frontend | React + Vite | TypeScript strict mode |
| Styling | Tailwind CSS | Use design tokens |
| Terminal | xterm.js | Standard addons only |
| SSH | russh (Rust) | Memory-safe implementation |
| Database | SQLite | Single file, no server |
| Encryption | AES-256-GCM | Argon2id key derivation |

### 6.2 Performance Constraints
- Cold start: < 2 seconds
- Memory idle: < 50MB
- Memory 5 tabs: < 100MB
- SSH overhead: < 5ms vs native
- Binary size: < 20MB

### 6.3 Security Constraints
- Credentials encrypted at rest
- No plaintext logging
- No telemetry/analytics
- No cloud dependency
- Zeroize credentials from memory

---

## 7. Assumptions

### 7.1 User Assumptions
- Users have basic SSH knowledge
- Users understand terminal commands
- Users have SSH servers to connect to
- Users prefer local-first applications

### 7.2 Technical Assumptions
- Target machines have SSH servers running
- Network connectivity available for SSH
- Sufficient disk space for SQLite database
- OS supports WebView2 (Windows) or equivalent

---

## 8. Dependencies

### 8.1 External Dependencies
| Dependency | Version | Purpose |
|------------|---------|---------|
| Tauri | v2.x | App framework |
| russh | Latest | SSH implementation |
| xterm.js | Latest | Terminal emulator |
| rusqlite | Latest | SQLite bindings |
| React | 18.x | UI framework |
| Vite | Latest | Build tool |
| Tailwind CSS | Latest | Styling |

### 8.2 Development Dependencies
| Dependency | Purpose |
|------------|---------|
| Node.js | Frontend development |
| Rust | Backend development |
| Cargo | Rust package manager |
| Bun | JavaScript runtime |
| Git | Version control |

---

## 9. Deliverables

### 9.1 MVP Deliverables
1. **Application Binary**
   - Windows MSI installer
   - macOS DMG installer
   - Linux AppImage

2. **Source Code**
   - Complete Tauri v2 project
   - All source files documented
   - Unit tests for critical paths

3. **Documentation**
   - README.md (setup instructions)
   - User guide
   - Developer documentation

### 9.2 Post-MVP Deliverables
1. Mobile apps (Android/iOS)
2. Multi-device sync
3. Advanced features

---

## 10. Timeline

| Phase | Duration | Deliverables |
|-------|----------|--------------|
| Phase 1: Setup | Week 1 | Project scaffold, database, basic UI |
| Phase 2: SSH | Week 2-3 | SSH backend, terminal integration |
| Phase 3: Hosts | Week 4 | Host management CRUD |
| Phase 4: Vault | Week 5 | Credential encryption |
| Phase 5: Features | Week 6-7 | Snippets, settings, SFTP, port forwarding |
| Phase 6: Polish | Week 8 | Testing, packaging, documentation |

**Total MVP Duration: 8 weeks**

---

## 11. Success Criteria

### 11.1 Functional Success
- [ ] Can add/edit/delete hosts
- [ ] Can connect to multiple SSH servers simultaneously
- [ ] Credentials stored securely
- [ ] Snippets work correctly
- [ ] SFTP file browser functional
- [ ] Port forwarding configurable

### 11.2 Technical Success
- [ ] App starts in < 2 seconds
- [ ] Memory usage within targets
- [ ] Works on Windows, macOS, Linux
- [ ] No crashes or data loss
- [ ] All tests passing

### 11.3 User Experience Success
- [ ] Clean, intuitive interface
- [ ] Keyboard shortcuts work
- [ ] Dark mode looks good
- [ ] Error messages helpful
- [ ] Responsive layout

---

*This document defines the complete scope of ShellMate MVP. Any features not listed here should be considered for post-MVP releases.*
