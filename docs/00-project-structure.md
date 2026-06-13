# Project Structure Plan
## ShellMate — SSH Client (v1.0 Production)

**Version:** 2.3
**Last Updated:** 2026-06-13

---

## 1. High-Level Directory Structure

```
shellmate/
├── .github/                    # GitHub Actions & workflows
│   ├── workflows/
│   │   ├── ci.yml              # Continuous integration
│   │   ├── release.yml         # Release automation
│   │   └── codeql.yml          # Security scanning
│   └── ISSUE_TEMPLATE/
│       ├── bug_report.md
│       └── feature_request.md
│
├── docs/                       # Project documentation
│   ├── architecture/
│   ├── frontend/
│   ├── backend/
│   ├── database/
│   ├── security/
│   └── devops/
│
├── src-tauri/                  # Rust backend (Tauri)
│   ├── src/
│   │   ├── main.rs             # Entry point
│   │   ├── lib.rs              # Library exports
│   │   ├── commands/           # Tauri command handlers
│   │   │   ├── mod.rs
│   │   │   ├── host.rs         # Host CRUD operations
│   │   │   ├── ssh.rs          # SSH connection management
│   │   │   ├── vault.rs        # Credential encryption/decryption
│   │   │   ├── snippet.rs      # Snippet CRUD operations
│   │   │   ├── sftp.rs         # SFTP file operations
│   │   │   ├── port_forward.rs # Port forwarding management
│   │   │   ├── credential.rs   # Credential management
│   │   │   ├── group.rs        # Group management
│   │   │   ├── system.rs       # System commands
│   │   │   ├── theme.rs        # Theme management
│   │   │   ├── known_hosts.rs  # SSH known hosts
│   │   │   ├── broadcast.rs    # Broadcast mode
│   │   │   ├── discovery.rs    # mDNS device discovery
│   │   │   ├── vip_access.rs   # VIP passwordless SSH access
│   │   │   ├── p2p_sync.rs     # P2P local sync engine
│   │   │   ├── settings.rs     # App settings management
│   │   │   ├── git.rs          # Git integration (branch, status)
│   │   │   └── history.rs      # Command history CRUD
│   │   │
│   │   ├── ssh/                # SSH implementation
│   │   │   ├── mod.rs
│   │   │   ├── connection.rs   # SSH connection handler
│   │   │   ├── session.rs      # SSH session management
│   │   │   ├── auth.rs         # Authentication methods
│   │   │   ├── channel.rs      # SSH channel management
│   │   │   └── keepalive.rs    # SSH keepalive implementation
│   │   │
│   │   ├── sftp/               # SFTP implementation
│   │   │   ├── mod.rs
│   │   │   ├── client.rs       # SFTP client
│   │   │   ├── operations.rs   # File operations (upload, download, etc.)
│   │   │   └── permissions.rs  # Permission handling
│   │   │
│   │   ├── db/                 # Database layer
│   │   │   ├── mod.rs
│   │   │   ├── models.rs       # Data models
│   │   │   ├── schema.rs       # Database schema
│   │   │   ├── migrations.rs   # Schema migrations
│   │   │   └── queries/        # Query modules
│   │   │       ├── mod.rs
│   │   │       ├── hosts.rs
│   │   │       ├── groups.rs
│   │   │       ├── credentials.rs
│   │   │       ├── snippets.rs
│   │   │       └── settings.rs
│   │   │
│   │   ├── crypto/             # Encryption/decryption
│   │   │   ├── mod.rs
│   │   │   ├── aes.rs          # AES-256-GCM implementation
│   │   │   ├── key_derivation.rs # Argon2id key derivation
│   │   │   └── vault.rs        # Vault operations
│   │   │
│   │   ├── errors.rs           # Error types and handling
│   │   ├── state.rs            # Application state management
│   │   └── utils.rs            # Utility functions
│   │
│   ├── migrations/             # SQLite migrations
│   │   ├── 001_initial_schema.sql
│   │   ├── 002_add_indexes.sql
│   │   └── ...
│   │
│   ├── Cargo.toml              # Rust dependencies
│   ├── build.rs                # Build script
│   ├── tauri.conf.json         # Tauri configuration
│   ├── capabilities/           # Tauri capabilities
│   │   └── default.json
│   ├── icons/                  # App icons
│   └── resources/              # Bundled resources
│
├── src/                        # React frontend
│   ├── main.tsx                # Entry point
│   ├── App.tsx                 # Root component
│   │
│   ├── components/             # UI components
│   │   ├── ui/                 # Base UI components (shadcn/ui)
│   │   │   ├── button.tsx
│   │   │   ├── input.tsx
│   │   │   ├── dialog.tsx
│   │   │   ├── dropdown-menu.tsx
│   │   │   ├── context-menu.tsx
│   │   │   ├── tabs.tsx
│   │   │   ├── tooltip.tsx
│   │   │   ├── separator.tsx
│   │   │   ├── scroll-area.tsx
│   │   │   └── ...
│   │   │
│   │   ├── layout/             # Layout components
│   │   │   ├── AppLayout.tsx   # Main app layout
│   │   │   ├── Sidebar.tsx     # Left sidebar
│   │   │   ├── ContentArea.tsx # Main content area (panel routing)
│   │   │   ├── TabBar.tsx      # Terminal tab bar
│   │   │   ├── StatusBar.tsx   # Bottom status bar
│   │   │   └── TitleBar.tsx    # Custom title bar
│   │   │
│   │   ├── terminal/           # Terminal components
│   │   │   ├── Terminal.tsx    # xterm.js wrapper
│   │   │   ├── TerminalTab.tsx # Single terminal tab
│   │   │   ├── TerminalManager.tsx # Multi-tab manager
│   │   │   └── TerminalSearch.tsx  # Terminal search bar
│   │   │
│   │   ├── hosts/              # Host management components
│   │   │   ├── HostList.tsx    # Host list view
│   │   │   ├── HostItem.tsx    # Single host item
│   │   │   ├── HostForm.tsx    # Add/Edit host form
│   │   │   ├── HostGroup.tsx   # Host group component
│   │   │   └── HostSearch.tsx  # Host search bar
│   │   │
│   │   ├── sftp/               # SFTP components
│   │   │   ├── SftpBrowser.tsx # SFTP file browser
│   │   │   ├── FileList.tsx    # File list view
│   │   │   ├── FileItem.tsx    # Single file item
│   │   │   ├── FileUpload.tsx  # Upload component
│   │   │   └── FileContextMenu.tsx # Right-click menu
│   │   │
│   │   ├── snippets/           # Snippet components
│   │   │   ├── SnippetList.tsx # Snippet list view
│   │   │   ├── SnippetItem.tsx # Single snippet item
│   │   │   ├── SnippetForm.tsx # Add/Edit snippet form
│   │   │   └── SnippetSearch.tsx # Snippet search
│   │   │
│   │   ├── settings/           # Settings components
│   │   │   ├── SettingsDialog.tsx # Settings modal
│   │   │   ├── GeneralSettings.tsx
│   │   │   ├── TerminalSettings.tsx
│   │   │   ├── SecuritySettings.tsx
│   │   │   └── ShortcutSettings.tsx
│   │   │
│   │   └── vault/              # Vault components
│   │       ├── VaultUnlock.tsx # Master password entry
│   │       ├── VaultSetup.tsx  # Initial vault setup
│   │       └── VaultLock.tsx   # Lock screen
│   │
│   │   ├── vip/                # VIP Access components
│   │   │   └── VipAccessPanel.tsx # VIP passwordless access config
│   │   │
│   │   └── sync/               # P2P Sync components
│   │       └── P2pSyncPanel.tsx # P2P local sync UI
│   │
│   ├── stores/                 # Zustand stores
│   │   ├── host-store.ts       # Host state management
│   │   ├── tab-store.ts        # Tab state management
│   │   ├── terminal-store.ts   # Terminal state
│   │   ├── vault-store.ts      # Vault state
│   │   ├── snippet-store.ts    # Snippet state
│   │   ├── sftp-store.ts       # SFTP state
│   │   ├── port-forward-store.ts # Port forwarding state
│   │   ├── settings-store.ts   # Settings state
│   │   ├── broadcast-store.ts  # Broadcast mode state
│   │   ├── discovery-store.ts  # mDNS discovery state
│   │   ├── ui-store.ts         # UI state (sidebar, modals, etc.)
│   │   ├── command-store.ts    # Command palette registry
│   │   ├── shortcuts-store.ts  # Keyboard shortcuts bindings
│   │   └── history-store.ts    # Command history state
│   │
│   ├── hooks/                  # Custom React hooks
│   │   ├── useTerminal.ts      # Terminal hook
│   │   ├── useSsh.ts           # SSH connection hook
│   │   ├── useSftp.ts          # SFTP operations hook
│   │   ├── useHosts.ts         # Host management hook
│   │   ├── useSnippets.ts      # Snippet management hook
│   │   ├── useVault.ts         # Vault operations hook
│   │   ├── useKeyboardShortcuts.ts # Keyboard shortcuts hook
│   │   ├── useCommands.ts      # Command palette registration
│   │   ├── useAutoUpdater.ts   # Auto-updater hook
│   │   └── useTheme.ts         # Theme management hook
│   │
│   ├── lib/                    # Utility libraries
│   │   ├── tauri.ts            # Tauri invoke wrappers
│   │   ├── constants.ts        # App constants
│   │   ├── validators.ts       # Form validation
│   │   └── formatters.ts       # Data formatters
│   │
│   ├── types/                  # TypeScript types
│   │   ├── host.ts             # Host types
│   │   ├── ssh.ts              # SSH types
│   │   ├── sftp.ts             # SFTP types
│   │   ├── snippet.ts          # Snippet types
│   │   ├── vault.ts            # Vault types
│   │   ├── settings.ts         # Settings types
│   │   └── index.ts            # Type exports
│   │
│   └── styles/                 # Global styles
│       ├── globals.css         # Global CSS
│       ├── terminal.css        # Terminal-specific styles
│       └── themes/             # Theme definitions
│           ├── dark.css
│           └── light.css
│
├── public/                     # Static assets
│   ├── fonts/                  # Custom fonts
│   │   ├── JetBrainsMono/
│   │   └── FiraCode/
│   └── icons/                  # Static icons
│
├── scripts/                    # Build & development scripts
│   ├── setup.sh                # Development setup
│   ├── build.sh                # Build script
│   └── release.sh              # Release script
│
├── tests/                      # Test files
│   ├── unit/                   # Unit tests
│   │   ├── backend/            # Rust unit tests
│   │   └── frontend/           # Frontend unit tests
│   ├── integration/            # Integration tests
│   └── e2e/                    # End-to-end tests
│
├── .env.example                # Environment variables template
├── .gitignore                  # Git ignore rules
├── .prettierrc                 # Prettier configuration
├── .eslintrc.js                # ESLint configuration
├── tailwind.config.js          # Tailwind CSS configuration
├── tsconfig.json               # TypeScript configuration
├── vite.config.ts              # Vite configuration
├── package.json                # Node.js dependencies
├── README.md                   # Project README
├── PRD.md                      # Product Requirements Document
├── CONTRIBUTING.md             # Contributing guidelines
└── LICENSE                     # Project license
```

---

## 2. Key Files Description

### 2.1 Configuration Files

| File | Purpose |
|------|---------|
| `tauri.conf.json` | Tauri app configuration (window size, permissions, etc.) |
| `Cargo.toml` | Rust dependencies and build configuration |
| `package.json` | Node.js dependencies and scripts |
| `vite.config.ts` | Vite bundler configuration |
| `tailwind.config.js` | Tailwind CSS customization |
| `tsconfig.json` | TypeScript compiler options |

### 2.2 Core Files

| File | Purpose |
|------|---------|
| `src-tauri/src/main.rs` | Rust entry point, app initialization |
| `src/main.tsx` | React entry point |
| `src/App.tsx` | Root React component, routing |

### 2.3 Data Flow Files

| File | Purpose |
|------|---------|
| `src/lib/tauri.ts` | Centralized Tauri invoke calls |
| `src/stores/*.ts` | State management (Zustand) |
| `src-tauri/src/commands/*.rs` | Backend command handlers |

---

## 3. Module Organization

### 3.1 Frontend Modules

```
src/
├── components/
│   ├── ui/          # Reusable UI primitives
│   ├── layout/      # App structure
│   ├── terminal/    # Core terminal feature
│   ├── hosts/       # Host management
│   ├── sftp/        # File browser
│   ├── snippets/    # Command snippets
│   ├── settings/    # App settings
│   └── vault/       # Security/encryption
│
├── stores/          # Global state
├── hooks/           # Business logic
├── lib/             # Utilities
├── types/           # Type definitions
└── styles/          # Styling
```

### 3.2 Backend Modules

```
src-tauri/src/
├── commands/        # API layer (Tauri commands)
│   ├── host.rs      # Host CRUD
│   ├── ssh.rs       # SSH connection management
│   ├── vault.rs     # Vault operations
│   ├── snippet.rs   # Snippet CRUD
│   ├── sftp.rs      # SFTP operations
│   ├── port_forward.rs # Port forwarding
│   ├── credential.rs # Credential management
│   ├── group.rs     # Group management
│   ├── system.rs    # System commands
│   ├── theme.rs     # Theme management
│   ├── known_hosts.rs # SSH known hosts
│   ├── broadcast.rs # Broadcast mode
│   ├── discovery.rs # mDNS device discovery
│   ├── vip_access.rs # VIP passwordless SSH access
│   ├── p2p_sync.rs  # P2P local sync engine
│   └── settings.rs  # App settings
├── ssh/             # SSH implementation (russh)
├── sftp/            # SFTP implementation
├── port_forward/    # SSH tunnel/port forwarding
├── db/              # SQLite database operations
├── crypto/          # AES-256-GCM + Argon2id encryption
├── vault/           # Master password + vault state
├── known_hosts/     # SSH host key verification
├── errors.rs        # Error handling (thiserror)
├── state.rs         # App state (AppState)
└── lib.rs           # Library exports, Tauri setup
```

---

## 4. Naming Conventions

### 4.1 Files & Folders
- **Components:** PascalCase (e.g., `HostForm.tsx`)
- **Hooks:** camelCase with `use` prefix (e.g., `useTerminal.ts`)
- **Stores:** kebab-case with `-store` suffix (e.g., `host-store.ts`)
- **Utilities:** camelCase (e.g., `validators.ts`)
- **Types:** camelCase (e.g., `host.ts`)
- **Rust files:** snake_case (e.g., `connection.rs`)

### 4.2 Exports
- **Components:** Named exports (e.g., `export function HostForm() {}`)
- **Hooks:** Named exports (e.g., `export function useTerminal() {}`)
- **Types:** Named exports with type keyword (e.g., `export type Host = {}`)
- **Utilities:** Named exports (e.g., `export function validateHost() {}`)

---

## 5. Import Order

```typescript
// 1. External libraries
import { useState } from 'react';
import { invoke } from '@tauri-apps/api/core';

// 2. Internal types
import type { Host, SSHConfig } from '@/types';

// 3. Internal stores
import { useHostStore } from '@/stores/host-store';

// 4. Internal hooks
import { useTerminal } from '@/hooks/useTerminal';

// 5. Internal components
import { Button } from '@/components/ui/button';
import { HostForm } from '@/components/hosts/HostForm';

// 6. Utilities
import { validateHost } from '@/lib/validators';

// 7. Styles
import './styles.css';
```

---

## 6. Asset Organization

### 6.1 Fonts
- Store in `public/fonts/`
- Use WOFF2 format for web fonts
- Include JetBrains Mono and Fira Code as defaults

### 6.2 Icons
- App icons in `src-tauri/icons/`
- UI icons use Lucide React library
- Custom SVGs in `public/icons/`

### 6.3 Images
- Minimal usage (prefer CSS/SVG)
- Store in `public/images/` if needed

---

*This document provides the complete project structure for ShellMate. Follow this structure when adding new files or modules.*
