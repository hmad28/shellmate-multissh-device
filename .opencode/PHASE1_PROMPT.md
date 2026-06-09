# ShellMate Phase 1 — Project Setup & Basic UI

## Objective
Initialize a Tauri v2 desktop app with React + Vite + TypeScript frontend and Rust backend. Set up SQLite database with schema migrations, and build the basic app layout (sidebar, tab bar, content area, status bar).

## Tech Stack
- **Framework:** Tauri v2
- **Frontend:** React 18 + Vite + TypeScript (strict mode)
- **Styling:** Tailwind CSS 3 + shadcn/ui components
- **State Management:** Zustand 4
- **Database:** SQLite via rusqlite
- **Package Manager:** Bun (frontend), Cargo (backend)

## Step-by-Step Implementation

### 1. Scaffold Tauri v2 Project
```bash
# Use Tauri v2 create command with React + Vite template
# Project name: shellmate
# Identifier: com.shellmate.app
```

If the directory already has files, merge into existing structure. Don't overwrite docs/ or PRD.md.

### 2. Configure Frontend
- Set up TypeScript with strict mode in `tsconfig.json`
- Install and configure Tailwind CSS 3:
  - Add custom dark theme colors (dark background: `#0a0a0f`, sidebar: `#111118`, accent: `#3b82f6`)
  - Configure JetBrains Mono as terminal font
- Install shadcn/ui and initialize:
  - Components needed: `button`, `input`, `dialog`, `dropdown-menu`, `context-menu`, `tabs`, `tooltip`, `separator`, `scroll-area`
- Configure Vite for Tauri v2 compatibility

### 3. Install Key Dependencies
**Frontend (bun add):**
```
@tauri-apps/api@^2
@tauri-apps/plugin-shell@^2
zustand@^4
@xterm/xterm@^5
@xterm/addon-fit
@xterm/addon-search
@xterm/addon-web-links
lucide-react
class-variance-authority
clsx
tailwind-merge
```

**Backend (cargo add in src-tauri/):**
```
rusqlite = { version = "0.31", features = ["bundled"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
uuid = { version = "1", features = ["v4"] }
chrono = { version = "0.4", features = ["serde"] }
thiserror = "1"
zeroize = { version = "1", features = ["derive"] }
aes-gcm = "0.10"
argon2 = "0.5"
rand = "0.8"
```

### 4. Database Setup (Rust Backend)
Create `src-tauri/src/db/` module:

**schema.rs** — Define all tables:
```sql
-- Groups
CREATE TABLE IF NOT EXISTS groups (
  id TEXT PRIMARY KEY,
  name TEXT NOT NULL,
  color TEXT,
  parent_id TEXT REFERENCES groups(id),
  sort_order INTEGER DEFAULT 0
);

-- Credentials (encrypted)
CREATE TABLE IF NOT EXISTS credentials (
  id TEXT PRIMARY KEY,
  type TEXT NOT NULL CHECK (type IN ('password', 'private_key')),
  encrypted_data BLOB NOT NULL,
  nonce BLOB NOT NULL,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

-- Hosts
CREATE TABLE IF NOT EXISTS hosts (
  id TEXT PRIMARY KEY,
  label TEXT NOT NULL,
  hostname TEXT NOT NULL,
  port INTEGER NOT NULL DEFAULT 22,
  username TEXT NOT NULL,
  auth_type TEXT NOT NULL CHECK (auth_type IN ('password', 'key', 'key_passphrase')),
  credential_id TEXT NOT NULL REFERENCES credentials(id),
  group_id TEXT REFERENCES groups(id),
  tags TEXT,
  notes TEXT,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

-- Snippets
CREATE TABLE IF NOT EXISTS snippets (
  id TEXT PRIMARY KEY,
  title TEXT NOT NULL,
  command TEXT NOT NULL,
  description TEXT,
  tags TEXT,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

-- Port Forwards
CREATE TABLE IF NOT EXISTS port_forwards (
  id TEXT PRIMARY KEY,
  host_id TEXT NOT NULL REFERENCES hosts(id),
  type TEXT NOT NULL CHECK (type IN ('local', 'remote')),
  local_port INTEGER NOT NULL,
  remote_host TEXT NOT NULL,
  remote_port INTEGER NOT NULL,
  enabled INTEGER NOT NULL DEFAULT 1
);

-- Settings
CREATE TABLE IF NOT EXISTS settings (
  key TEXT PRIMARY KEY,
  value TEXT NOT NULL
);
```

**migrations.rs** — Simple migration runner that checks applied migrations and runs new ones.

**mod.rs** — Database initialization:
- Create/open SQLite database at OS-appropriate path
- Run migrations on startup
- Expose connection via AppState

### 5. App State (Rust)
Create `src-tauri/src/state.rs`:
```rust
pub struct AppState {
    pub db: Mutex<Connection>,
    // Future: vault key, SSH sessions, etc.
}
```

### 6. Basic Tauri Commands
Create placeholder commands in `src-tauri/src/commands/`:
- `host.rs` — `create_host`, `get_hosts`, `update_host`, `delete_host` (stub implementations that return placeholder data or basic CRUD)
- `settings.rs` — `get_settings`, `update_setting`

### 7. Frontend Layout Components

**AppLayout.tsx** — Main layout:
```
┌──────────────┬────────────────────────────────────┐
│   Sidebar    │         Main Content Area          │
│  (250px)     │  ┌──┬──┬──┬──────────────────┐    │
│              │  │T1│T2│T3│         +        │    │
│              │  └──┴──┴──┴──────────────────┘    │
│              │                                    │
│              │   [Terminal / Content]             │
│              │                                    │
│              ├────────────────────────────────────┤
│              │ Status Bar                         │
└──────────────┴────────────────────────────────────┘
```

**Sidebar.tsx:**
- Search bar at top (placeholder)
- "Hosts" section with expand/collapse groups
- "Add Host" button at bottom
- Snippets link
- Settings link (gear icon)

**TabBar.tsx:**
- Horizontal tab list
- "+" button to add new tab
- Close button (x) per tab
- Tab states: connected (green dot), connecting (yellow), disconnected (red)
- Drag-and-drop reorder (basic)

**StatusBar.tsx:**
- Left: connection info or "Ready"
- Right: vault status (locked/unlocked), app version

**TitleBar.tsx:**
- Custom title bar with app name "ShellMate"
- Window controls (minimize, maximize, close)

### 8. Zustand Stores

**tab-store.ts:**
```typescript
interface Tab {
  id: string;
  hostId: string | null;
  label: string;
  status: 'connected' | 'connecting' | 'disconnected';
}

interface TabStore {
  tabs: Tab[];
  activeTabId: string | null;
  addTab: (hostId?: string) => void;
  closeTab: (id: string) => void;
  setActiveTab: (id: string) => void;
  updateTabStatus: (id: string, status: Tab['status']) => void;
}
```

**host-store.ts:**
```typescript
interface Host {
  id: string;
  label: string;
  hostname: string;
  port: number;
  username: string;
  authType: 'password' | 'key' | 'key_passphrase';
  groupId: string | null;
  tags: string[];
  notes: string;
}

interface HostStore {
  hosts: Host[];
  groups: Group[];
  loadHosts: () => Promise<void>;
  addHost: (host: HostInput) => Promise<void>;
  updateHost: (id: string, host: HostInput) => Promise<void>;
  deleteHost: (id: string) => Promise<void>;
}
```

**ui-store.ts:**
```typescript
interface UiStore {
  sidebarCollapsed: boolean;
  activePanel: 'hosts' | 'snippets' | 'settings';
  toggleSidebar: () => void;
  setActivePanel: (panel: string) => void;
}
```

### 9. Global Styles
- Dark theme by default
- CSS variables for colors
- Terminal-specific styles for xterm.js container
- Smooth transitions for sidebar collapse/expand

### 10. Tauri Configuration
Update `tauri.conf.json`:
- Set window title to "ShellMate"
- Set window size: 1200x800 default
- Set min size: 800x600
- Enable decorations: false (for custom title bar)
- Set app identifier: com.shellmate.app

## Deliverables Checklist
- [ ] Tauri v2 app launches and shows window
- [ ] Dark themed layout renders (sidebar + content + status bar)
- [ ] SQLite database creates on first launch with all tables
- [ ] Zustand stores created and connected
- [ ] Tab bar renders with "+" button
- [ ] Sidebar renders with placeholder groups
- [ ] Custom title bar displays "ShellMate"
- [ ] TypeScript strict mode, no errors
- [ ] All Rust code compiles without warnings
- [ ] `cargo build` succeeds
- [ ] `bun run build` succeeds

## Notes
- Keep implementations minimal but functional — this is scaffolding, not final features
- Use placeholder data where needed (e.g., sample hosts in sidebar)
- Don't implement SSH yet — just the UI shell
- Don't implement encryption yet — just the database schema
- Focus on clean architecture that's easy to extend in Phase 2
