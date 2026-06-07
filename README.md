# ShellMate

**Codename:** ShellMate *(working title)*
**Version:** 1.1
**Status:** Draft

## Overview

ShellMate is a self-hosted, local-first SSH client desktop app with modern UX, multi-SSH connection support, and multi-device capability.

## Features

- **Multi-SSH Connection** - Connect to multiple SSH servers simultaneously
- **Multi-Device Support** - Desktop (Windows, macOS, Linux) + Mobile (Post-MVP)
- **Local-First** - All data stored locally, no cloud dependency
- **Encrypted Vault** - AES-256-GCM encryption for credentials
- **Modern UI** - Clean, minimal, keyboard-first design

## Tech Stack

| Layer | Technology |
|-------|-----------|
| App Framework | Tauri v2 |
| Frontend | React + Vite |
| Styling | Tailwind CSS |
| Terminal | xterm.js |
| SSH Backend | Rust (russh crate) |
| Local Storage | SQLite |
| Encryption | AES-256-GCM |

## Documentation

See [PRD.md](PRD.md) for full product requirements.

## License

TBD
