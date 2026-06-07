# Frontend Plan
## ShellMate - React + Vite + Tailwind CSS

**Version:** 1.0
**Last Updated:** 2026-06-07

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
│ │    Hosts     │                                    │
│ │  + Add Host  │   [Terminal / SFTP / Settings]     │
│ │              │                                    │
│ │  [Snippets]  │                                    │
│ │  [Settings]  │                                    │
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

## 8. Accessibility

### 8.1 ARIA Labels
- All interactive elements have ARIA labels
- Tab navigation supported
- Screen reader friendly

### 8.2 Keyboard Navigation
- Full keyboard navigation
- Focus indicators
- Skip links

### 8.3 Color Contrast
- WCAG 2.1 AA compliance
- High contrast mode support

---

## 9. Testing Strategy

### 9.1 Unit Tests
- Component rendering
- Store logic
- Hook behavior
- Utility functions

### 9.2 Integration Tests
- Terminal connection flow
- Host CRUD operations
- Vault unlock/lock flow

### 9.3 E2E Tests (Post-MVP)
- Full SSH connection
- Multi-tab operations
- SFTP file operations

---

*This document outlines the complete frontend architecture and implementation plan for ShellMate.*
