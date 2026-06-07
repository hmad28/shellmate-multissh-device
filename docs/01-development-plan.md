# Development Plan
## ShellMate - SSH Client Desktop App

**Version:** 1.0
**Last Updated:** 2026-06-07
**Timeline:** 8 weeks (MVP Desktop)

---

## 1. Development Overview

### 1.1 Methodology
- **Approach:** Iterative development with weekly milestones
- **Version Control:** Git with feature branch workflow
- **Code Reviews:** Required for all merges to main
- **Testing:** Unit tests for critical paths, integration tests for SSH

### 1.2 Team Structure
- **Primary Developer:** Matt (Full-stack)
- **AI Assistant:** Mimo 2.5 (Code generation, debugging, documentation)

### 1.3 Development Environment
- **OS:** Windows 11 (primary), macOS/Linux (cross-platform testing)
- **IDE:** VS Code / Cursor / OpenCode
- **Terminal:** Termul Manager / PowerShell / Git Bash
- **Package Manager:** Bun (frontend), Cargo (backend)

---

## 2. Phase 1: Project Setup (Week 1)

### 2.1 Day 1-2: Scaffold Project
**Tasks:**
- [ ] Initialize Tauri v2 project with React + Vite template
- [ ] Configure Tailwind CSS with custom theme
- [ ] Set up TypeScript strict mode
- [ ] Configure ESLint + Prettier
- [ ] Initialize Git repository with .gitignore
- [ ] Create README.md and basic documentation

**Deliverables:**
- Working Tauri app that opens a window
- Basic development environment configured
- Git repository with initial commit

### 2.2 Day 3-4: Database Setup
**Tasks:**
- [ ] Add rusqlite dependency to Cargo.toml
- [ ] Create database schema (SQLite)
- [ ] Implement database initialization
- [ ] Create migration system
- [ ] Set up database path per OS

**Deliverables:**
- SQLite database created on app start
- Schema migrations running successfully
- Database module ready for queries

### 2.3 Day 5-7: Basic UI Layout
**Tasks:**
- [ ] Create main app layout (sidebar + content area)
- [ ] Implement custom title bar
- [ ] Create sidebar with host list placeholder
- [ ] Add tab bar for terminal sessions
- [ ] Implement status bar
- [ ] Set up Zustand stores (host, tab, UI)

**Deliverables:**
- Responsive app layout
- Sidebar with placeholder content
- Tab bar ready for terminal tabs
- Basic state management working

---

## 3. Phase 2: Core SSH Feature (Week 2-3)

### 3.1 Week 2: SSH Backend
**Tasks:**
- [ ] Add russh dependency to Cargo.toml
- [ ] Implement SSH connection handler
- [ ] Add password authentication
- [ ] Add SSH key authentication
- [ ] Implement SSH session management
- [ ] Add SSH keepalive support
- [ ] Create Tauri commands for SSH operations

**Deliverables:**
- SSH connection to remote server working
- Password and key auth implemented
- Session management functional
- Tauri commands ready for frontend

### 3.2 Week 3: Terminal Integration
**Tasks:**
- [ ] Add xterm.js to frontend
- [ ] Create Terminal component wrapper
- [ ] Implement SSH ↔ Terminal data streaming
- [ ] Add terminal resize support
- [ ] Implement copy/paste functionality
- [ ] Add terminal search (xterm.js addon)
- [ ] Create multi-tab terminal manager

**Deliverables:**
- Interactive SSH terminal working
- Multi-tab sessions functional
- Terminal resize and copy/paste working
- Basic terminal features complete

---

## 4. Phase 3: Host Management (Week 4)

### 4.1 Week 4: Host CRUD
**Tasks:**
- [ ] Create Host data model (Rust + TypeScript)
- [ ] Implement host CRUD in backend
- [ ] Create HostList component
- [ ] Create HostForm component (add/edit)
- [ ] Add host validation
- [ ] Implement host groups
- [ ] Add drag-and-drop for host organization
- [ ] Create host search functionality

**Deliverables:**
- Add, edit, delete hosts working
- Host groups with expand/collapse
- Host search functional
- Data persisted to SQLite

---

## 5. Phase 4: Vault & Security (Week 5)

### 5.1 Week 5: Credential Encryption
**Tasks:**
- [ ] Implement Argon2id key derivation
- [ ] Add AES-256-GCM encryption
- [ ] Create vault unlock/setup flow
- [ ] Implement credential storage
- [ ] Add auto-lock after idle
- [ ] Implement manual lock (Ctrl+L)
- [ ] Add master password change
- [ ] Secure memory handling (zeroize)

**Deliverables:**
- Credentials encrypted at rest
- Vault lock/unlock working
- Auto-lock after idle
- Secure credential retrieval

---

## 6. Phase 5: Advanced Features (Week 6-7)

### 6.1 Week 6: Snippets & Settings
**Tasks:**
- [ ] Create Snippet data model
- [ ] Implement snippet CRUD
- [ ] Create SnippetList component
- [ ] Add snippet execution to terminal
- [ ] Implement snippet search
- [ ] Create Settings dialog
- [ ] Add terminal settings (font, size, cursor)
- [ ] Add theme settings (dark/light)

**Deliverables:**
- Snippet management working
- Snippets executable to terminal
- Settings dialog functional
- Theme switching working

### 6.2 Week 7: SFTP & Port Forwarding
**Tasks:**
- [ ] Implement SFTP client in Rust
- [ ] Create SFTP browser UI
- [ ] Add file upload/download
- [ ] Implement file operations (rename, delete, mkdir)
- [ ] Add drag-and-drop upload
- [ ] Implement port forwarding rules
- [ ] Add port forwarding status display
- [ ] Create port conflict detection

**Deliverables:**
- SFTP file browser working
- File upload/download functional
- Port forwarding configurable
- Port conflicts detected

---

## 7. Phase 6: Polish & Release (Week 8)

### 7.1 Week 8: Polish & Packaging
**Tasks:**
- [ ] Add keyboard shortcuts
- [ ] Implement error handling & toast notifications
- [ ] Add reconnect UI for disconnected sessions
- [ ] Create onboarding flow
- [ ] Add export/import hosts (JSON)
- [ ] Performance optimization
- [ ] Cross-platform testing (Windows, macOS, Linux)
- [ ] Create installers (MSI, DMG, AppImage)
- [ ] Write user documentation
- [ ] Prepare release notes

**Deliverables:**
- All keyboard shortcuts working
- Error handling throughout app
- Cross-platform builds working
- Installers created
- Documentation complete

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
| Scope creep | High | Strict MVP scope, defer non-essential features |
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
- [ ] All MVP features complete
- [ ] Cross-platform testing passed
- [ ] Installers created and tested
- [ ] Documentation complete
- [ ] Release notes written

---

## 11. Communication

### 11.1 Daily
- Progress updates in Telegram
- Blockers identified immediately

### 11.2 Weekly
- Sprint review with Mimo
- Demo of completed features
- Planning for next week

---

*This development plan provides a structured approach to building ShellMate MVP within 8 weeks. Adjust timeline as needed based on progress.*
