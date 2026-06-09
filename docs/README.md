# ShellMate Documentation Index
## Complete Project Documentation

**Version:** 1.2
**Last Updated:** 2026-06-09
**Status:** Approved for development

---

## 📋 Documentation Overview

This document provides an index of all planning and architecture documentation for ShellMate, a self-hosted, local-first SSH client desktop application.

---

## 📚 Documentation List

### Core Planning Documents

| # | Document | Description | Status |
|---|----------|-------------|--------|
| 00 | [Project Structure](./00-project-structure.md) | Complete directory structure and file organization | ✅ Complete |
| 01 | [Development Plan](./01-development-plan.md) | 8-week development timeline and milestones | ✅ Complete |
| 02 | [Project Scope](./02-project-scope.md) | MVP features, goals, and constraints | ✅ Complete |
| 03 | [Frontend Plan](./03-frontend-plan.md) | React + Vite + Tailwind architecture | ✅ Complete |
| 04 | [Backend Plan](./04-backend-plan.md) | Rust + Tauri backend implementation | ✅ Complete |
| 05 | [ERD Plan](./05-erd-plan.md) | Database schema and relationships | ✅ Complete |
| 06 | [Architecture Plan](./06-architecture-plan.md) | System architecture and data flows | ✅ Complete |
| 07 | [Security Plan](./07-security-plan.md) | Encryption, authentication, and security | ✅ Complete |
| 08 | [DevOps Plan](./08-devops-plan.md) | CI/CD, testing, and deployment | ✅ Complete |

---

## 🎯 Quick Start Guide

### For Developers
1. Read [Project Structure](./00-project-structure.md) to understand the codebase
2. Follow [Development Plan](./01-development-plan.md) for implementation timeline
3. Review [Frontend Plan](./03-frontend-plan.md) or [Backend Plan](./04-backend-plan.md) based on your focus

### For Product Managers
1. Start with [Project Scope](./02-project-scope.md) for feature overview
2. Review [Development Plan](./01-development-plan.md) for timeline
3. Check [ERD Plan](./05-erd-plan.md) for data model understanding

### For Security Review
1. Review [Security Plan](./07-security-plan.md) for encryption and security practices
2. Check [Architecture Plan](./06-architecture-plan.md) for trust boundaries
3. Review [Backend Plan](./04-backend-plan.md) for credential handling

---

## 🏗️ Architecture Summary

```
┌─────────────────────────────────────────────────────────┐
│                    ShellMate Architecture                 │
├─────────────────────────────────────────────────────────┤
│                                                          │
│  Frontend (React + Vite + Tailwind)                      │
│  ├── Terminal (xterm.js)                                 │
│  ├── Host Management                                     │
│  ├── SFTP Browser                                        │
│  └── Settings UI                                         │
│                                                          │
│  Tauri Bridge (invoke / events)                          │
│                                                          │
│  Backend (Rust)                                          │
│  ├── SSH Handler (russh)                                 │
│  ├── SFTP Client                                         │
│  ├── Vault (AES-256-GCM)                                 │
│  └── Database (SQLite)                                   │
│                                                          │
│  Security                                                │
│  ├── Argon2id Key Derivation                             │
│  ├── AES-256-GCM Encryption                              │
│  ├── Zeroize Memory Protection                           │
│  └── SSH Host Verification                               │
│                                                          │
└─────────────────────────────────────────────────────────┘
```

---

## 📊 Development Timeline

| Week | Phase | Deliverables |
|------|-------|--------------|
| 1 | Setup | Project scaffold, database, basic UI |
| 2-3 | SSH | SSH backend, terminal integration |
| 4 | Hosts | Host management CRUD |
| 5 | Vault | Credential encryption |
| 6-7 | Features | Snippets, settings, SFTP, port forwarding |
| 8 | Polish | Testing, packaging, documentation |

**Total MVP Duration: 8 weeks**

---

## 🔐 Security Highlights

- **Encryption:** AES-256-GCM for credentials at rest
- **Key Derivation:** Argon2id (memory-hard, brute-force resistant)
- **Memory Safety:** Zeroize sensitive data after use
- **No Telemetry:** All data stays local
- **SSH Verification:** Host key verification enabled

---

## 🛠️ Tech Stack

| Layer | Technology |
|-------|------------|
| Framework | Tauri v2 |
| Frontend | React + Vite + TypeScript |
| Styling | Tailwind CSS |
| Terminal | xterm.js |
| SSH | russh (Rust) |
| Database | SQLite |
| Encryption | AES-256-GCM + Argon2id |

---

## 📝 Next Steps

1. **Review Documentation** - Read through all planning documents
2. **Setup Development Environment** - Follow DevOps Plan setup
3. **Start Phase 1** - Begin project scaffolding
4. **Iterate** - Follow development plan milestones

---

## 📞 Contact

- **Project:** ShellMate
- **Author:** Matt
- **License:** MIT

---

*This index provides a complete overview of all ShellMate planning documentation.*
