# Frontend Plan
## ShellMate - React + Vite + Tailwind CSS

**Version:** 1.1
**Last Updated:** 2026-06-09

---

## 1. Frontend Architecture

### 1.1 Technology Stack
| Technology | Version | Purpose |
|------------|---------|---------|
| React | 18.x | UI framework |
| TypeScript | 5.x | Type safety |
| Vite | Latest | Build tool |
| Tailwind CSS | 3.x | Styling |
| Zustand | 4.x | State management |
| xterm.js | 5.x | Terminal emulator |

### 1.2 Directory Structure
```
src/
├── main.tsx              # Entry point
├── App.tsx               # Root component
│
├── components/           # UI components
│   ├── ui/               # Base components
│   ├── layout/           # App structure
│   ├── terminal/         # Terminal
│   ├── hosts/            # Host management
│   ├── sftp/             # File browser
│   ├── snippets/         # Snippets
│   ├── settings/         # Settings
│   └── vault/            # Vault UI
│
├── stores/               # Zustand stores
├── hooks/                # Custom hooks
├── lib/                  # Utilities
├── types/                # TypeScript types
└── styles/               # Global styles
```

---

## 2. Component Architecture

### 2.1 Layout Components

#### AppLayout.tsx
```typescript
// Main app layout structure
// Props: none
// Children: Sidebar, TabBar, ContentArea

interface AppLayoutProps {
  children: React.ReactNode;
}

// Structure:
// ┌──────────────┬────────────────────────────────────┐
// │   Sidebar    │         Main Content Area          │
// │              │  ┌──┬──┬──┬──────────────────┐    │
// │  [Search]    │  │T1│T2│T3│         +        │    │
// │  ▼ Groups    │  └──┴──┴──┴──────────────────┘    │
// │    Hosts     │                                    │
// │  + Add Host  │   [Terminal / SFTP / Settings]     │
// │              │                                    │
// │  [Snippets]  │                                    │
// │  [Settings]  │                                    │
// └──────────────┴────────────────────────────────────┘
```

#### Sidebar.tsx
```typescript
// Left sidebar with host list and navigation
// Props: none (uses stores)

// Features:
// - Search bar for hosts
// - Host groups (expandable)
// - Host list
// - Add host button
// - Snippets link
// - Settings link
```

#### TabBar.tsx
```typescript
// Terminal tab bar
// Props: none (uses tab-store)

// Features:
// - List of open tabs
// - Add new tab button
// - Close tab button
// - Tab reorder (drag-and-drop)
// - Connection status indicator
// - Tab color by group
```

### 2.2 Terminal Components

#### Terminal.tsx
```typescript
// xterm.js wrapper component
// Props: TerminalProps

interface TerminalProps {
  sessionId: string;
  hostId: string;
  onConnectionChange: (status: ConnectionStatus) => void;
}

// Features:
// - Initialize xterm.js instance
// - Handle SSH data streaming
// - Support terminal resize
// - Copy/paste functionality
// - Terminal search
// - Cleanup on unmount
```

#### TerminalManager.tsx
```typescript
// Multi-tab terminal manager
// Props: none (uses stores)

// Features:
// - Render active terminal
// - Handle tab switching
// - Manage terminal instances
// - Lazy loading of inactive tabs
```

### 2.3 Host Components

#### HostList.tsx
```typescript
// List of SSH hosts grouped by category
// Props: none (uses host-store)

// Features:
// - Render host groups
// - Render hosts within groups
// - Expand/collapse groups
// - Host selection
// - Context menu (edit, delete, connect)
```

#### HostForm.tsx
```typescript
// Add/Edit host form
// Props: HostFormProps

interface HostFormProps {
  host?: Host;           // Existing host for edit mode
  onSubmit: (host: HostInput) => void;
  onCancel: () => void;
}

// Fields:
// - Label (display name)
// - Hostname (IP/domain)
// - Port (default: 22)
// - Username
// - Auth type (password/key/key+passphrase)
// - Credential input
// - Group selection
// - Tags
// - Notes
```

### 2.4 SFTP Components

#### SftpBrowser.tsx
```typescript
// SFTP file browser panel
// Props: SftpBrowserProps

interface SftpBrowserProps {
  hostId: string;
  sessionId: string;
}

// Features:
// - Directory listing
// - File navigation
// - Upload files
// - Download files
// - File operations (rename, delete, mkdir)
// - Progress indicator
```

### 2.5 Vault Components

#### VaultUnlock.tsx
```typescript
// Master password entry screen
// Props: VaultUnlockProps

interface VaultUnlockProps {
  onUnlock: (password: string) => void;
  error?: string;
}

// Features:
// - Password input
// - Show/hide password
// - Unlock button
// - Error display
```

---

## 3. State Management

### 3.1 Host Store (host-store.ts)
```typescript
interface HostState {
  hosts: Host[];
  groups: HostGroup[];
  selectedHostId: string | null;
  
  // Actions
  addHost: (host: HostInput) => Promise<void>;
  updateHost: (id: string, host: Partial<HostInput>) => Promise<void>;
  deleteHost: (id: string) => Promise<void>;
  selectHost: (id: string | null) => void;
  
  // Groups
  addGroup: (group: GroupInput) => Promise<void>;
  updateGroup: (id: string, group: Partial<GroupInput>) => Promise<void>;
  deleteGroup: (id: string) => Promise<void>;
  
  // Loading
  loadHosts: () => Promise<void>;
}
```

### 3.2 Tab Store (tab-store.ts)
```typescript
interface TabState {
  tabs: Tab[];
  activeTabId: string | null;
  
  // Actions
  openTab: (hostId: string) => string;  // Returns tab ID
  closeTab: (tabId: string) => void;
  setActiveTab: (tabId: string) => void;
  reorderTabs: (fromIndex: number, toIndex: number) => void;
  
  // Tab operations
  getTab: (tabId: string) => Tab | undefined;
  getTabsByHost: (hostId: string) => Tab[];
}

interface Tab {
  id: string;
  hostId: string;
  label: string;
  status: ConnectionStatus;
  createdAt: number;
}
```

### 3.3 Vault Store (vault-store.ts)
```typescript
interface VaultState {
  isUnlocked: boolean;
  isSetup: boolean;
  lockTimeout: number;  // minutes
  
  // Actions
  setup: (masterPassword: string) => Promise<void>;
  unlock: (masterPassword: string) => Promise<boolean>;
  lock: () => void;
  changePassword: (oldPassword: string, newPassword: string) => Promise<void>;
  
  // Credential operations
  getCredential: (id: string) => Promise<string>;
  saveCredential: (id: string, data: string) => Promise<string>;
}
```

### 3.4 Settings Store (settings-store.ts)
```typescript
interface SettingsState {
  theme: 'dark' | 'light' | 'system';
  fontFamily: string;
  fontSize: number;
  cursorStyle: 'block' | 'bar' | 'underline';
  cursorBlink: boolean;
  scrollbackLines: number;
  autoLockTimeout: number;  // minutes
  keepaliveInterval: number;  // seconds
  
  // Actions
  updateTheme: (theme: Theme) => void;
  updateTerminal: (settings: TerminalSettings) => void;
  updateSecurity: (settings: SecuritySettings) => void;
}
```

---

## 4. Custom Hooks

### 4.1 useTerminal.ts
```typescript
// Hook for terminal management
function useTerminal(sessionId: string, hostId: string) {
  // Initialize xterm.js
  // Handle SSH data streaming
  // Support resize
  // Copy/paste
  // Cleanup
  
  return {
    terminalRef: React.RefObject<HTMLDivElement>,
    isConnected: boolean,
    connect: () => Promise<void>,
    disconnect: () => void,
    resize: (cols: number, rows: number) => void,
  };
}
```

### 4.2 useSsh.ts
```typescript
// Hook for SSH operations
function useSsh() {
  // Connect to host
  // Disconnect
  // Send data
  // Receive data
  
  return {
    connect: (hostId: string) => Promise<string>,  // Returns session ID
    disconnect: (sessionId: string) => void,
    send: (sessionId: string, data: string) => void,
    onOutput: (sessionId: string, callback: (data: string) => void) => void,
  };
}
```

### 4.3 useVault.ts
```typescript
// Hook for vault operations
function useVault() {
  // Unlock vault
  // Get/set credentials
  // Auto-lock
  
  return {
    isUnlocked: boolean,
    unlock: (password: string) => Promise<boolean>,
    lock: () => void,
    getCredential: (id: string) => Promise<string>,
    saveCredential: (data: string) => Promise<string>,
  };
}
```

---

## 5. Styling System

### 5.1 Design Tokens
```css
/* Tailwind CSS custom theme */
@layer base {
  :root {
    /* Colors */
    --color-primary: #3b82f6;
    --color-secondary: #6b7280;
    --color-success: #22c55e;
    --color-warning: #f59e0b;
    --color-error: #ef4444;
    
    /* Spacing */
    --spacing-xs: 4px;
    --spacing-sm: 8px;
    --spacing-md: 16px;
    --spacing-lg: 24px;
    --spacing-xl: 32px;
    
    /* Typography */
    --font-family: 'JetBrains Mono', monospace;
    --font-size-sm: 12px;
    --font-size-md: 14px;
    --font-size-lg: 16px;
    
    /* Border radius */
    --radius-sm: 4px;
    --radius-md: 8px;
    --radius-lg: 12px;
  }
}
```

### 5.2 Component Variants
```typescript
// Using class-variance-authority for variants
import { cva } from 'class-variance-authority';

const buttonVariants = cva(
  'inline-flex items-center justify-center rounded-md font-medium',
  {
    variants: {
      variant: {
        primary: 'bg-primary text-white hover:bg-primary/90',
        secondary: 'bg-secondary text-white hover:bg-secondary/90',
        ghost: 'hover:bg-secondary/10',
      },
      size: {
        sm: 'h-8 px-3 text-sm',
        md: 'h-10 px-4 text-base',
        lg: 'h-12 px-6 text-lg',
      },
    },
    defaultVariants: {
      variant: 'primary',
      size: 'md',
    },
  }
);
```

---

## 6. Keyboard Shortcuts

### 6.1 Shortcut Implementation
```typescript
// hooks/useKeyboard.ts
const shortcuts: Record<string, () => void> = {
  'ctrl+t': () => openNewTab(),
  'ctrl+w': () => closeActiveTab(),
  'ctrl+tab': () => switchToNextTab(),
  'ctrl+shift+tab': () => switchToPrevTab(),
  'ctrl+k': () => openSnippetPanel(),
  'ctrl+shift+f': () => openSftpBrowser(),
  'ctrl+,': () => openSettings(),
  'ctrl+l': () => lockVault(),
  'ctrl+f': () => searchTerminal(),
};

// Use event listener or library like hotkeys-js
```

### 6.2 Shortcut Reference
| Shortcut | Action |
|----------|--------|
| `Ctrl+T` | New terminal tab |
| `Ctrl+W` | Close current tab |
| `Ctrl+Tab` | Next tab |
| `Ctrl+Shift+Tab` | Previous tab |
| `Ctrl+1-9` | Switch to tab N |
| `Ctrl+K` | Open snippets |
| `Ctrl+Shift+F` | Open SFTP |
| `Ctrl+,` | Open settings |
| `Ctrl+L` | Lock vault |
| `Ctrl+F` | Search terminal |
| `Ctrl+Shift+C` | Copy (terminal) |
| `Ctrl+Shift+V` | Paste (terminal) |

---

## 7. Performance Optimization

### 7.1 Lazy Loading
- Lazy load inactive terminal tabs
- Load SFTP browser on demand
- Code splitting for settings/dialogs

### 7.2 Memoization
- Memoize host list items
- Memoize terminal output
- Use React.memo for pure components

### 7.3 Virtualization
- Virtualize long host lists
- Virtualize terminal output (if needed)

### 7.4 Bundle Optimization
- Tree shaking for unused code
- Dynamic imports for heavy components
- Optimize Tailwind CSS output

---

## 8. Accessibility (a11y)

### 8.1 Standards
- Target **WCAG 2.1 AA** compliance for non-terminal UI
- Terminal content (xterm.js) is exempt from typical text contrast checks but UI chrome around it is not

### 8.2 Semantic HTML & ARIA
- Use semantic elements: `<button>`, `<nav>`, `<main>`, `<aside>`, `<dialog>`
- All icon-only buttons must have `aria-label` (e.g. close tab, add host)
- Tab list uses `role="tablist"`, tabs use `role="tab"` with `aria-selected`
- Dialogs use `role="dialog"` with `aria-modal="true"` and labeled by title
- Status indicators (Connected/Connecting/Disconnected) announced via `aria-live="polite"`
- shadcn/ui components are built on Radix UI which provides ARIA out of the box — leverage this

### 8.3 Keyboard Navigation
- Every action reachable via keyboard (no mouse-only)
- Logical tab order through sidebar → tab bar → terminal
- `Esc` closes dialogs and dropdowns
- Focus trap inside open dialogs (shadcn/ui handles this)
- Focus restored to trigger element when dialog closes
- Skip link "Skip to terminal" at top for screen reader users
- Visible focus rings (Tailwind `focus-visible:ring-2`)

### 8.4 Color & Contrast
- Verify contrast ratio ≥ 4.5:1 for normal text, ≥ 3:1 for large text and UI components
- Don't rely on color alone — connection status uses color **and** icon (●/○/✗)
- Test dark and light themes separately
- Tools: axe DevTools, Lighthouse, Stark plugin

### 8.5 Motion & Animation
- Respect `prefers-reduced-motion` — disable transitions for tab switching, sidebar collapse, modal animations
- No flashing/strobing content (epilepsy safety)

### 8.6 Screen Reader Considerations
- Terminal output **not** announced (would be overwhelming) — but provide a "Copy last output" shortcut
- Connection state changes announced via `aria-live` region in status bar
- Vault lock state changes announced

### 8.7 Touch Targets (forward-compat for mobile)
- Min 44x44px hit area for interactive elements
- Adequate spacing between adjacent buttons

### 8.8 Testing
- Manual: keyboard-only navigation walkthrough each release
- Automated: axe-core via Vitest + Testing Library
- Periodic: NVDA (Windows) and VoiceOver (macOS) smoke test

---

## 9. Internationalization (i18n)

### 9.1 MVP Scope
- **Default language: English** (UI strings)
- Code Bahasa Indonesia comments allowed in source for team productivity
- All user-facing strings extracted to constants from day one (no hardcoded strings in JSX)

### 9.2 Architecture
- Lightweight approach for MVP: single `src/i18n/en.ts` with typed string keys
- No runtime translation library yet (avoid bundle bloat)
- Structure ready for `react-i18next` or `next-intl` swap-in post-MVP

```typescript
// src/i18n/en.ts
export const strings = {
  app: {
    name: 'ShellMate',
    locked: 'Vault Locked',
  },
  hosts: {
    add: 'Add Host',
    edit: 'Edit Host',
    delete: 'Delete Host',
    confirmDelete: 'Are you sure you want to delete this host?',
  },
  terminal: {
    connecting: 'Connecting...',
    connected: 'Connected',
    disconnected: 'Disconnected',
    reconnect: 'Reconnect',
  },
  vault: {
    unlock: 'Unlock Vault',
    masterPasswordPlaceholder: 'Enter master password',
    setupWarning: 'If you forget your master password, your data cannot be recovered.',
  },
} as const;

// Usage:
import { strings } from '@/i18n/en';
<button>{strings.hosts.add}</button>
```

### 9.3 Post-MVP Plan
- Bahasa Indonesia translation (`src/i18n/id.ts`)
- Locale detection from OS / user setting
- Plural forms via ICU MessageFormat or `react-i18next`
- Date/time formatting via `Intl.DateTimeFormat`
- RTL support deferred until needed

### 9.4 Rules for Developers
- ❌ Never inline user-facing strings: `<button>Add Host</button>`
- ✅ Always use the strings object: `<button>{strings.hosts.add}</button>`
- Error messages from backend should also be string keys, not raw text

---

## 10. Testing Strategy

### 10.1 Unit Tests
- Component rendering
- Store logic
- Hook behavior
- Utility functions

### 10.2 Integration Tests
- Terminal connection flow
- Host CRUD operations
- Vault unlock/lock flow

### 10.3 Accessibility Tests
- axe-core integration via Vitest
- Manual keyboard navigation per release

### 10.4 E2E Tests (Post-MVP)
- Full SSH connection
- Multi-tab operations
- SFTP file operations

---

*This document outlines the complete frontend architecture and implementation plan for ShellMate.*
