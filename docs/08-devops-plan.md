# DevOps Plan
## ShellMate - Development Operations

**Version:** 1.0
**Last Updated:** 2026-06-07

---

## 1. Development Environment

### 1.1 Required Tools
| Tool | Version | Purpose |
|------|---------|---------|
| Node.js | 20.x LTS | Frontend development |
| Bun | 1.x | JavaScript runtime |
| Rust | Latest stable | Backend development |
| Cargo | Latest | Rust package manager |
| Git | Latest | Version control |
| VS Code | Latest | IDE |

### 1.2 VS Code Extensions
```json
{
  "recommendations": [
    "rust-lang.rust-analyzer",
    "tauri-apps.tauri-vscode",
    "bradlc.vscode-tailwindcss",
    "dbaeumer.vscode-eslint",
    "esbenp.prettier-vscode",
    "ms-vscode.vscode-typescript-next"
  ]
}
```

### 1.3 Setup Script
```bash
#!/bin/bash
# scripts/setup.sh

echo "🚀 Setting up ShellMate development environment..."

# Check Node.js
if ! command -v node &> /dev/null; then
    echo "❌ Node.js not found. Please install Node.js 20.x LTS"
    exit 1
fi

# Check Rust
if ! command -v rustc &> /dev/null; then
    echo "📦 Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
    source ~/.cargo/env
fi

# Check Bun
if ! command -v bun &> /dev/null; then
    echo "📦 Installing Bun..."
    curl -fsSL https://bun.sh/install | bash
fi

# Install frontend dependencies
echo "📦 Installing frontend dependencies..."
bun install

# Install Tauri CLI
echo "📦 Installing Tauri CLI..."
cargo install tauri-cli

echo "✅ Setup complete!"
echo ""
echo "To start development:"
echo "  bun run dev"
```

---

## 2. Build Process

### 2.1 Build Commands
```json
{
  "scripts": {
    "dev": "tauri dev",
    "build": "tauri build",
    "build:debug": "tauri build --debug",
    "lint": "eslint src --ext .ts,.tsx",
    "lint:fix": "eslint src --ext .ts,.tsx --fix",
    "format": "prettier --write \"src/**/*.{ts,tsx,css}\"",
    "test": "vitest",
    "test:coverage": "vitest --coverage",
    "typecheck": "tsc --noEmit"
  }
}
```

### 2.2 Build Pipeline
```
┌──────────────┐     ┌──────────────┐     ┌──────────────┐
│    Source    │     │    Build     │     │   Output     │
│    Code      │     │   Process    │     │   Artifacts  │
└──────┬───────┘     └──────┬───────┘     └──────┬───────┘
       │                    │                    │
       │ 1. Git clone       │                    │
       │───────────────────>│                    │
       │                    │ 2. bun install     │
       │                    │ 3. cargo build     │
       │                    │ 4. tauri build     │
       │                    │───────────────────>│
       │                    │                    │
       │                    │                    │ 5. Artifacts
       │                    │                    │    - Windows: .msi
       │                    │                    │    - macOS: .dmg
       │                    │                    │    - Linux: .AppImage
```

### 2.3 Platform-Specific Builds
```bash
# Windows
bun run build:tauri:win

# macOS (Apple Silicon)
bun run build:tauri:mac-arm

# macOS (Intel)
bun run build:tauri:mac-x64

# Linux
bun run build:tauri:linux

# Debug build (faster compilation)
bun run build:tauri:debug
```

---

## 3. Testing Strategy

### 3.1 Test Types
```
┌─────────────────────────────────────────────────────────┐
│                    Testing Pyramid                        │
├─────────────────────────────────────────────────────────┤
│                                                          │
│  ┌─────────────────────────────────────────────────┐   │
│  │  E2E Tests (Playwright)                          │   │
│  │  - Full user workflows                           │   │
│  │  - Cross-browser testing                         │   │
│  │  - Performance testing                           │   │
│  └─────────────────────────────────────────────────┘   │
│                                                          │
│  ┌─────────────────────────────────────────────────┐   │
│  │  Integration Tests (Vitest)                      │   │
│  │  - API integration                               │   │
│  │  - Database operations                           │   │
│  │  - Component integration                         │   │
│  └─────────────────────────────────────────────────┘   │
│                                                          │
│  ┌─────────────────────────────────────────────────┐   │
│  │  Unit Tests (Vitest + cargo test)                │   │
│  │  - Component rendering                           │   │
│  │  - Business logic                                │   │
│  │  - Utility functions                             │   │
│  └─────────────────────────────────────────────────┘   │
│                                                          │
└─────────────────────────────────────────────────────────┘
```

### 3.2 Test Commands
```bash
# Frontend tests
bun run test
bun run test:coverage

# Backend tests
cd src-tauri && cargo test

# All tests
bun run test && cd src-tauri && cargo test
```

---

## 4. CI/CD Pipeline

### 4.1 GitHub Actions Workflow
```yaml
# .github/workflows/ci.yml
name: CI

on:
  push:
    branches: [main, develop]
  pull_request:
    branches: [main]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Setup Node.js
        uses: actions/setup-node@v4
        with:
          node-version: '20'
          
      - name: Setup Bun
        uses: oven-sh/setup-bun@v1
          
      - name: Setup Rust
        uses: dtolnay/rust-toolchain@stable
          
      - name: Install dependencies
        run: bun install
          
      - name: Lint
        run: bun run lint
        
      - name: Type check
        run: bun run typecheck
        
      - name: Test frontend
        run: bun run test
        
      - name: Test backend
        run: cd src-tauri && cargo test
```

### 4.2 Release Workflow
```yaml
# .github/workflows/release.yml
name: Release

on:
  push:
    tags:
      - 'v*'

jobs:
  release:
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
    steps:
      - uses: actions/checkout@v4
      
      - name: Setup environment
        # ... setup steps
      
      - name: Build
        run: bun run build
        
      - name: Upload artifacts
        uses: actions/upload-artifact@v4
        with:
          name: release-${{ matrix.os }}
          path: src-tauri/target/release/bundle/
```

---

## 5. Code Quality

### 5.1 Linting Configuration
```javascript
// .eslintrc.js
module.exports = {
  extends: [
    'eslint:recommended',
    'plugin:react/recommended',
    'plugin:react-hooks/recommended',
    'plugin:@typescript-eslint/recommended',
  ],
  parser: '@typescript-eslint/parser',
  plugins: ['react', 'react-hooks', '@typescript-eslint'],
  settings: {
    react: {
      version: 'detect',
    },
  },
  rules: {
    '@typescript-eslint/no-unused-vars': 'error',
    '@typescript-eslint/no-explicit-any': 'warn',
    'react-hooks/rules-of-hooks': 'error',
    'react-hooks/exhaustive-deps': 'warn',
  },
};
```

### 5.2 Formatting Configuration
```json
// .prettierrc
{
  "semi": true,
  "trailingComma": "es5",
  "singleQuote": true,
  "printWidth": 80,
  "tabWidth": 2,
  "useTabs": false
}
```

### 5.3 Pre-commit Hooks
```bash
# Install husky
bun add -D husky

# Setup pre-commit hook
bunx husky add .husky/pre-commit "bun run lint && bun run typecheck"
```

---

## 6. Version Control

### 6.1 Git Workflow
```
┌─────────────────────────────────────────────────────────┐
│                    Git Workflow                           │
├─────────────────────────────────────────────────────────┤
│                                                          │
│  main (production)                                       │
│      ↑                                                   │
│      │                                                   │
│  develop (staging)                                       │
│      ↑                                                   │
│      │                                                   │
│  feature/* (development)                                 │
│      ↑                                                   │
│      │                                                   │
│  Developer                                                │
│                                                          │
└─────────────────────────────────────────────────────────┘
```

### 6.2 Branch Naming
```
feature/ssh-connection
feature/host-management
bugfix/tab-close-crash
hotfix/security-patch
release/v1.0.0
```

### 6.3 Commit Messages
```
feat: add SSH key authentication
fix: resolve tab close crash
docs: update API documentation
style: format code with prettier
refactor: extract SSH connection logic
test: add unit tests for vault
chore: update dependencies
```

---

## 7. Documentation

### 7.1 Documentation Structure
```
docs/
├── README.md                 # Project overview
├── CONTRIBUTING.md           # Contributing guidelines
├── CHANGELOG.md              # Version history
├── SECURITY.md               # Security policy
├── architecture/
│   ├── overview.md           # Architecture overview
│   ├── frontend.md           # Frontend architecture
│   └── backend.md            # Backend architecture
├── api/
│   ├── ssh.md                # SSH API documentation
│   └── sftp.md               # SFTP API documentation
└── user-guide/
    ├── installation.md       # Installation guide
    ├── getting-started.md    # Quick start guide
    └── features.md           # Feature documentation
```

### 7.2 API Documentation
- Rust doc comments (`///`)
- TypeScript JSDoc comments
- Auto-generated API docs

---

## 8. Monitoring & Logging

### 8.1 Application Logging
```rust
use log::{info, warn, error, debug};

pub fn setup_logging() {
    env_logger::Builder::from_env(
        env_logger::Env::default().default_filter_or("info")
    )
    .init();
}

// Usage
info!("SSH connection established to {}", host);
warn!("Connection timeout for session {}", session_id);
error!("Failed to connect: {}", error);
debug!("Sending data: {} bytes", data.len());
```

### 8.2 Error Tracking
- Local error logging (no external services)
- Error categorization
- User-friendly error messages

---

## 9. Release Process

### 9.1 Release Steps
1. Update version in `Cargo.toml` and `package.json`
2. Update `CHANGELOG.md`
3. Create git tag: `git tag v1.0.0`
4. Push tag: `git push origin v1.0.0`
5. CI/CD builds and creates release
6. Upload artifacts to GitHub Releases

### 9.2 Version Numbering
- **Major:** Breaking changes
- **Minor:** New features (backward compatible)
- **Patch:** Bug fixes

Example: `v1.2.3`
- 1 = Major
- 2 = Minor
- 3 = Patch

---

## 10. Backup & Recovery

### 10.1 Backup Strategy
- **Source Code:** Git repository (GitHub)
- **Database:** User copies file manually
- **Configuration:** Included in database

### 10.2 Recovery Process
1. Clone repository
2. Install dependencies
3. Build application
4. User restores database backup

---

*This document outlines the complete DevOps strategy and practices for ShellMate.*
