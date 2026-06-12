# Frontend Plan
## ShellMate — React + Vite + Tailwind (v1.0 Production)

**Version:** 2.3
**Last Updated:** 2026-06-11

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
- Structure ready for `react-i18next` or `next-intl` swap-in post-1.0

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

### 9.3 Post-1.0 Plan
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

### 10.4 E2E Tests (Phase 14)
- Full SSH connection
- Multi-tab operations
- SFTP file operations

---

## 11. Mobile UX Architecture (Phase 10)

### 11.1 Adaptive Layout

```typescript
// src/hooks/useFormFactor.ts
export function useFormFactor(): 'desktop' | 'tablet' | 'mobile' {
  const [w, h] = useViewport();
  if (w < 600) return 'mobile';
  if (w < 1024) return 'tablet';
  return 'desktop';
}
```

Layout components branch on form factor at the layout root, not deep in tree, to keep components reusable.

### 11.2 Mobile-Specific Components

```
src/components/mobile/
├── MobileLayout.tsx           # Bottom-sheet nav, full-screen content
├── ExtendedKeyBar.tsx         # Esc, Tab, Ctrl, Alt, ↑↓←→, |, ~, -, /
├── MobileTabSwitcher.tsx      # Swipeable tab list
├── BottomSheetSidebar.tsx     # Hosts list as bottom sheet
├── MobileSftpModal.tsx        # Full-screen SFTP browser
└── TouchTerminal.tsx          # Terminal with pinch-to-zoom + tap-to-show-toolbar
```

### 11.3 Gesture Handling

| Gesture | Action |
|---------|--------|
| Swipe left/right on terminal | Switch tabs |
| Pinch on terminal | Adjust font size |
| Long-press tab | Show context menu (close, broadcast toggle) |
| Pull-down on host list | Refresh / sync now |
| Two-finger tap | Toggle extended key bar visibility |

### 11.4 Background Lifecycle

- iOS: app backgrounding → kept alive briefly via Background Modes (audio session trick) to allow short reconnect window
- Android: foreground service for active SSH sessions (if user enables in settings)
- Show notification when session disconnects in background

### 11.5 Form Factor Test Matrix

- Phone portrait: iPhone SE, iPhone 15, Pixel 7
- Phone landscape: same devices rotated
- Tablet: iPad mini, iPad Pro 11"
- Foldable: galaxy fold open/closed (best-effort)

---

## 12. Theme System (Phase 4)

### 12.1 Token Pipeline

```
ThemeDefinition (TS interface)
       │
       ▼ apply()
   document.documentElement.style.setProperty('--bg', ...)
       │
       ▼
   Tailwind reads via theme('colors.bg.DEFAULT')
       │
       ▼
   Components render with correct theme
```

### 12.2 Theme Components

```
src/components/settings/themes/
├── ThemePicker.tsx        # Grid of preview tiles (built-in + custom)
├── ThemeEditor.tsx        # Live preview + token editors
├── ColorPicker.tsx        # HEX / HSL color input with eyedropper
├── TerminalPreview.tsx    # Sample terminal output with current palette
├── ImportThemeButton.tsx
└── ExportThemeButton.tsx
```

### 12.3 Storage

- Built-in themes: `src/themes/builtin/{dark,light,high-contrast}.ts`
- Custom themes: SQLite `themes` table, synced via Phase 9 sync engine
- Plugin-shipped themes: registered via `plugin.register_theme(...)` API

### 12.4 Theme File Format

```json
{
  "id": "ocean-dark",
  "name": "Ocean Dark",
  "base": "dark",
  "ui": { "bg": "#0a1929", "fg": "#e6f1ff", ... },
  "terminal": {
    "background": "#0a1929",
    "foreground": "#e6f1ff",
    "cursor": "#82aaff",
    "ansi": ["#000", "#ff5874", ..., "#fff"]
  },
  "fontFamily": "JetBrains Mono"
}
```

---

## 13. Broadcast Mode UI (Phase 6)

### 13.1 Components

```
src/components/broadcast/
├── BroadcastToolbar.tsx     # Toggle button + target chips at top of content area
├── BroadcastInput.tsx       # Single input field that fans out to all targets
├── BroadcastTargetChip.tsx  # Tab indicator with remove button
└── DangerCommandPrompt.tsx  # Confirm before broadcasting destructive commands
```

### 13.2 Behavior

- Toggle in tab right-click menu: "Add to broadcast"
- Broadcasted tabs show distinct colored border
- Single input bar appears at top of content area when ≥1 broadcast target active
- Input fans out via `tauri.ssh.send` to each target session id in parallel

### 13.3 Safety

Configurable list of dangerous patterns (default: `rm -rf`, `dd `, `mkfs`, `:(){`):
- Match against input before broadcast
- Show confirmation dialog with the command + list of targets
- User must explicitly confirm

---

## 14. Sync UI (Phase 9)

### 14.1 Components

```
src/components/sync/
├── SyncStatusIndicator.tsx   # In status bar: last sync, pending count, error icon
├── SyncSettings.tsx          # Backend selector, OAuth flow, credentials form
├── SyncDiagnostic.tsx        # Logs, last error, force re-sync, force re-encrypt
├── SelectiveSyncTree.tsx     # Hosts/snippets with checkbox to opt-in
└── ConflictResolutionModal.tsx  # Side-by-side merge UI for conflicts
```

### 14.2 Backend Setup Flow

1. Settings → Sync → Choose backend
2. OAuth consent (GDrive/Dropbox) or credentials form (S3/WebDAV/HTTP)
3. Test connection → success message
4. Initial upload → progress with cancel option
5. Sync now active

### 14.3 Conflict UI

Side-by-side compare with field-level diff:
- Local version on left, remote on right
- Field-by-field "use local" / "use remote" / "merged" buttons
- For host config conflicts: show all changed fields
- For group structure conflicts: tree-diff visualization

---

## 15. Plugin UI (Phase 12)

### 15.1 Components

```
src/components/plugins/
├── PluginManager.tsx         # List installed + enable/disable/uninstall
├── PluginInstallDialog.tsx   # Review manifest, capabilities, signature
├── CapabilityList.tsx        # Renders capability badges with descriptions
├── PluginPanel.tsx           # Wraps WASM-rendered custom panel
└── SecretAccessPrompt.tsx    # Per-access prompt when plugin requests vault read
```

### 15.2 Install Flow

1. Click "Load plugin from file..."
2. Open file picker → select `.wasm`
3. Read manifest, display:
   - Plugin name, author, version
   - Signature verification status
   - Required capabilities (with risk-level color coding)
4. User reviews + clicks Approve or Cancel
5. On Approve: instantiate sandbox, register hooks/panels
6. On Cancel: discard

### 15.3 Capability UX

| Capability | Color | Description shown |
|-----------|-------|-------------------|
| log | gray | "Write to plugin log only" |
| panel | blue | "Add a custom panel to the UI" |
| terminal_data | yellow | "See and modify what you type and what servers send back" |
| network | orange | "Make HTTP requests to: {allow-list}" |
| filesystem | orange | "Read/write files in {scoped path}" |
| secrets | red | "Read your vault credentials (asks every time)" |

---

## 16. Audit Log UI (Phase 13)

### 16.1 Components

```
src/components/audit/
├── AuditLogViewer.tsx        # Filterable timeline view
├── AuditEventCard.tsx        # Single event with metadata
├── AuditFilterBar.tsx        # Filter by host, date range, event type
├── AuditExportButton.tsx     # Export to signed JSONL
└── AuditRetentionSettings.tsx
```

### 16.2 Privacy UX

- Warning when enabling command history capture: "Commands you type may include sensitive data"
- Redaction patterns editor (regex with test input)
- Bulk delete option

---

## 17. Team Vault UI (Phase 11)

### 17.1 Components

```
src/components/team/
├── TeamSetup.tsx             # Create new team, generate master key
├── TeamInvitation.tsx        # Generate invite payload, scan/paste accept
├── TeamMemberList.tsx        # List + revoke
├── ShareHostDialog.tsx       # Pick hosts to share, set permissions
└── SharedHostsBadge.tsx      # Visual indicator that a host is team-shared
```

---

*This document outlines the complete frontend architecture and implementation plan for ShellMate.*
