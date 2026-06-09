# Changelog

All notable changes to ShellMate will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added — Phase 1: Project Setup (2026-06-09)

- Tauri v2 project scaffold with React 18 + Vite 6 + TypeScript (strict mode)
- Tailwind CSS 3 with custom dark theme palette (bg, sidebar, panel, elevated, border, fg, accent, status colors)
- SQLite database with full schema migrations:
  - Tables: `groups`, `credentials`, `hosts`, `snippets`, `port_forwards`, `settings`
  - WAL journal mode, foreign keys enforced, parameterized queries
  - Migration runner with `_migrations` tracking table
- Tauri commands: `app_version`, `get_hosts`, `create_host`, `update_host`, `delete_host`, `get_settings`, `set_setting`
- Application state with `parking_lot::Mutex<Connection>`
- Serializable `AppError` type via `thiserror`
- Layout components:
  - `AppLayout` — root layout shell
  - `TitleBar` — custom title bar with drag region and window controls
  - `Sidebar` — host list with search, groups, action buttons
  - `TabBar` — multi-tab session bar with status indicators
  - `StatusBar` — vault state and active connection display with `aria-live`
  - `ContentArea` — placeholder for terminal/SFTP/settings panels
- Zustand stores:
  - `tab-store` — multi-tab session state
  - `ui-store` — sidebar, panel, vault state
  - `host-store` — host CRUD with backend sync
- Typed Tauri invoke wrapper (`src/lib/tauri.ts`)
- i18n string module (`src/i18n/en.ts`) — English default, ready for post-MVP translation
- ESLint + Prettier + tsconfig strict mode
- MIT LICENSE
- Tauri icon set generated from custom SVG logo
- `rust-toolchain.toml` pinned to MSVC stable

### Verified
- `npm run typecheck` — pass
- `npm run lint` — pass
- `npm run build` — pass (197 KB / 62 KB gzipped, well within 500 KB budget)
- `cargo build` — pass (1 ringan unused-variant warning untuk forward-compat error variant)

### Decided During Phase 1
- Package manager: **npm** (Bun on local system was POSIX-only binary)
- Rust toolchain: **MSVC** (mingw-gnu had "export ordinal too large" linker error with Tauri)
- Window decorations: disabled, custom title bar implementation
- Frontend bundle baseline: 62 KB gzipped — performance budget headroom for Phase 2-6
