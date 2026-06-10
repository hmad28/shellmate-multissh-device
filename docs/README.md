# ShellMate Documentation Index

**Version:** 2.0
**Last Updated:** 2026-06-10
**Status:** In development (Phase 1-2 complete, scope expanded to full v1.0 production)

---

## Documentation Overview

This document provides an index of all planning and architecture documentation for ShellMate — a self-hosted, local-first SSH client targeting desktop and mobile, with optional E2E-encrypted multi-device sync.

ShellMate v1.0 is a **production release**, not an MVP. Scope spans 14 phases delivered scope-driven (no fixed timeline) — each phase ships when its acceptance criteria are met.

---

## Documentation List

| # | Document | Description |
|---|----------|-------------|
| 00 | [Project Structure](./00-project-structure.md) | Directory structure and file organization |
| 01 | [Development Plan](./01-development-plan.md) | 14 scope-driven phases with progress tracker |
| 02 | [Project Scope](./02-project-scope.md) | v1.0 features, goals, constraints, success criteria |
| 03 | [Frontend Plan](./03-frontend-plan.md) | React + Vite + Tailwind, mobile UX, theme system, broadcast UI |
| 04 | [Backend Plan](./04-backend-plan.md) | Rust + Tauri backend modules (ssh, mosh, sync, plugin, audit, team, biometric) |
| 05 | [ERD Plan](./05-erd-plan.md) | SQLite + SQLCipher schema and relationships |
| 06 | [Architecture Plan](./06-architecture-plan.md) | System architecture, multi-platform, sync, plugins, audit, themes, broadcast |
| 07 | [Security Plan](./07-security-plan.md) | Defense-in-depth encryption, biometric, sync security, plugin sandbox, team vault, audit |
| 08 | [DevOps Plan](./08-devops-plan.md) | CI/CD, testing, code signing, mobile builds, plugin distribution, security audit |

---

## Quick Start

### For Developers
1. Read [Project Structure](./00-project-structure.md) to understand the codebase
2. Check [Development Plan §0 Progress Tracker](./01-development-plan.md) to see what's done and what's next
3. Review [Frontend Plan](./03-frontend-plan.md) or [Backend Plan](./04-backend-plan.md) based on your focus
4. See `../README.md` for build commands

### For Product / Stakeholders
1. Start with [Project Scope](./02-project-scope.md) for feature inventory
2. Review [Development Plan](./01-development-plan.md) for phasing
3. Check [PRD.md](../PRD.md) §11 Resolved Decisions for trade-off rationale

### For Security Review
1. Review [Security Plan](./07-security-plan.md) — encryption, biometric, plugin sandbox, sync, team vault
2. Check [Architecture Plan §4 Security Architecture](./06-architecture-plan.md) for trust boundaries
3. Review [Backend Plan §10](./04-backend-plan.md) for credential handling and module structure

---

## Architecture Summary

```
+--------------------------------------------------+
|             Frontend (React + WebView)           |
|  Terminal | Hosts | SFTP | Settings | Plugins UI |
+----------------------+---------------------------+
                       | invoke / events
+----------------------v---------------------------+
|              Backend (Rust + Tauri)              |
|  ssh | mosh | sftp | vault | sync | plugin       |
|  audit | team | biometric | crypto | db          |
+----------------------+---------------------------+
                       |
        +--------------+----------------+
        |                               |
   SQLite + SQLCipher           Remote SSH / Mosh
        |
        +-- E2E-encrypted sync to user's own cloud
            (iCloud / GDrive / Dropbox / S3 / WebDAV)
```

---

## Phase Progress

| Phase | Status | Area |
|-------|--------|------|
| 1 | Complete (2026-06-09) | Project Setup |
| 2 | Complete (2026-06-10) | Core SSH + Vault + Terminal |
| 3 | Complete (2026-06-10) | Host Management & Persistence |
| 4 | Pending | Productivity & Settings (snippets, themes, shortcuts) |
| 5 | Pending | File Transfer & Network (SFTP, port forward) |
| 6 | Pending | Network Hardening (known hosts, Mosh, broadcast) |
| 7 | Pending | Full-DB Encryption (SQLCipher) |
| 8 | Pending | Biometric Unlock |
| 9 | Pending | Multi-Device Sync (E2E) |
| 10 | Pending | Mobile Apps (Android, iOS) |
| 11 | Pending | Team Vault |
| 12 | Pending | Plugin System (Wasmtime) |
| 13 | Pending | Audit Log |
| 14 | Pending | Polish & Distribution |

---

## Security Highlights

- **Encryption:** AES-256-GCM per-credential + SQLCipher full-DB (defense in depth)
- **Key Derivation:** Argon2id (memory-hard, OWASP params)
- **Memory Safety:** Zeroize sensitive data after use
- **Master Password:** length-first policy (NIST SP 800-63B), no recovery
- **Biometric Unlock:** Touch ID, Face ID, Windows Hello, Android BiometricPrompt
- **No Telemetry:** All data stays local; sync uses user's own cloud
- **SSH Host Verification:** TOFU + warning on key mismatch (Phase 6)
- **Plugin Sandbox:** Wasmtime with capability-based permissions

---

## Tech Stack

| Layer | Technology |
|-------|------------|
| App Framework | Tauri v2 (desktop + mobile) |
| Frontend | React 18 + Vite + TypeScript (strict) |
| Styling | Tailwind CSS 3 + shadcn/ui |
| Terminal | xterm.js |
| SSH | russh (Rust) |
| Mosh | Rust (Phase 6) |
| Local Storage | SQLite + SQLCipher (Phase 7) |
| Encryption | AES-256-GCM + Argon2id |
| Plugin Runtime | Wasmtime (WASM sandbox, Phase 12) |
| State | Zustand |

---

## Contact

- **Project:** ShellMate
- **Author:** Matt
- **License:** MIT

---

*This index provides a complete overview of all ShellMate planning documentation.*
