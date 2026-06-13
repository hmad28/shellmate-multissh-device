# DevOps Plan
## ShellMate — Development Operations (v1.0 Production)

**Version:** 2.3
**Last Updated:** 2026-06-11

---

## 1. Development Environment

### 1.1 Required Tools
| Tool | Version | Purpose |
|------|---------|---------|
| Node.js | 20.x LTS | Frontend development |
| npm | 10.x | Package manager |
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

# Install frontend dependencies
echo "📦 Installing frontend dependencies..."
npm install

# Install Tauri CLI
echo "📦 Installing Tauri CLI..."
cargo install tauri-cli

echo "✅ Setup complete!"
echo ""
echo "To start development:"
echo "  npm run tauri:dev"
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
       │                    │ 2. npm install     │
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
npm run build:tauri:win

# macOS (Apple Silicon)
npm run build:tauri:mac-arm

# macOS (Intel)
npm run build:tauri:mac-x64

# Linux
npm run build:tauri:linux

# Debug build (faster compilation)
npm run build:tauri:debug
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
npm run test
npm run test:coverage

# Backend tests
cd src-tauri && cargo test

# All tests
npm run test && cd src-tauri && cargo test
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
          
      - name: Setup Rust
        uses: dtolnay/rust-toolchain@stable
          
      - name: Install dependencies
        run: npm ci

      - name: Lint
        run: npm run lint
        
      - name: Type check
        run: npm run typecheck
        
      - name: Test frontend
        run: npm run test
        
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
        run: npm run tauri:build
        
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

### 13. Performance Budget Enforcement
```bash
# Install husky
npm add -D husky

# Setup pre-commit hook
npx husky add .husky/pre-commit "npm run lint && npm run typecheck"
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

## 11. Code Signing & Distribution

### 11.1 v1.0 Production Requirement

ShellMate v1.0 **must** ship signed binaries. Unsigned builds are dev-only.

| Platform | Cost | Tool | Required for v1.0 |
|----------|------|------|-------------------|
| **macOS** | Apple Developer Program: $99/year | `codesign` + `notarytool` | ✅ |
| **Windows** | Code Signing Cert: $200-500/year (EV: $300-600/year) | `signtool.exe` via Tauri bundler | ✅ |
| **Linux** | Free (GPG) | Sign AppImage with GPG | ✅ |
| **Android** | Free (self-sign) for direct APK; $25 one-time for Play Store | `apksigner` / Play Console | ✅ |
| **iOS** | Apple Developer Program (same $99/yr) | Xcode signing + App Store Connect | ✅ |

### 11.2 Pre-1.0 Dev Builds

Pre-release dev builds are unsigned. Users see OS warnings. Dev docs include workarounds:
- macOS: `xattr -d com.apple.quarantine ShellMate.app`
- Windows: SmartScreen "More info → Run anyway"
- Linux: AppImage works as-is
- Android: enable "Install unknown apps"

### 11.3 Tauri v2 Updater (Phase 14)

Auto-updater is part of v1.0:
- Generate Tauri updater key pair (`tauri signer generate`)
- Public key embedded in app, private key in CI secret only
- Updates served from GitHub Releases or self-hosted endpoint
- Update channel: stable + beta (opt-in)
- Updates are signed even if app binary signing somehow fails (defense in depth)

### 11.4 Mobile Distribution

| Channel | Strategy |
|---------|----------|
| Android Play Store | Signed AAB upload, staged rollout (1% → 10% → 100%) |
| Android direct APK | GPG-signed download from GitHub Releases |
| iOS App Store | TestFlight beta then App Store review |
| F-Droid (optional) | Reproducible build, separate package |

### 11.5 Release Checksums

Every release includes:
- SHA-256 checksums for all artifacts
- GPG-signed checksum file
- Tauri updater manifest (signed)

---

## 12. Accessibility Testing

### 12.1 Automated
- **axe-core** integration via Vitest + Testing Library
- Run on every PR via CI
- Fail build on critical/serious violations
- Generate accessibility report per release

### 12.2 Manual
- Keyboard-only navigation walkthrough each release (no mouse)
- Screen reader smoke test: NVDA on Windows, VoiceOver on macOS
- Test with `prefers-reduced-motion` and high contrast OS settings
- Color contrast verification with axe DevTools and Lighthouse

### 12.3 Tooling
```bash
# Install
npm install -D @axe-core/react vitest-axe

# Example test (vitest)
import { render } from '@testing-library/react';
import { axe, toHaveNoViolations } from 'vitest-axe';

expect.extend({ toHaveNoViolations });

it('host form has no a11y violations', async () => {
  const { container } = render(<HostForm />);
  const results = await axe(container);
  expect(results).toHaveNoViolations();
});
```

---

## 13. Performance Budget Enforcement

### 13.1 Targets (from PRD)

| Metric | Target | Enforcement |
|--------|--------|-------------|
| Cold start | < 2 seconds | Manual benchmark per release |
| Memory idle | < 50 MB | Manual + heap profiling |
| Memory 5 tabs | < 100 MB | Manual benchmark |
| SSH overhead | < 5 ms vs native | Manual benchmark |
| Binary size | < 20 MB installer | **CI assertion** (block PR if exceeded) |
| Frontend bundle | < 500 KB gzipped | **CI assertion** (`vite-bundle-analyzer`) |

### 13.2 CI Size Gate

```yaml
# .github/workflows/ci.yml — additional step
- name: Check binary size
  run: |
    SIZE=$(stat -c%s src-tauri/target/release/bundle/appimage/*.AppImage)
    MAX=20971520  # 20 MB
    if [ $SIZE -gt $MAX ]; then
      echo "❌ Binary size $SIZE exceeds budget $MAX"
      exit 1
    fi

- name: Check frontend bundle size
  run: npm run build && npx vite-bundle-visualizer --output dist/stats.html
```

### 13.3 Profiling Tools

| What | Tool |
|------|------|
| Rust startup | `cargo flamegraph`, `tracing` spans |
| Rust memory | `dhat-rs`, `heaptrack` |
| Frontend bundle | `vite-bundle-visualizer`, `source-map-explorer` |
| Frontend runtime | Chrome DevTools (WebView2 supports it on Windows) |
| End-to-end startup | Custom wall-clock measurement in `main.rs` |

### 13.4 Benchmark Suite (Phase 14)

Pre-release benchmark script that reports:
- Cold start time (median of 5 runs)
- Memory at 0/1/5/10 tabs
- SSH connect time to known test server
- Frontend bundle size & install size

Results committed to `benchmarks/` for regression tracking.

---

## 14. Test Strategy Detail

### 14.1 Coverage Targets

| Layer | Target |
|-------|--------|
| Rust crypto module | 95%+ (critical) |
| Rust SSH module | 80%+ |
| Rust db module | 85%+ |
| Frontend stores | 80%+ |
| Frontend components | 60%+ (focus on logic, not styling) |
| Overall | 70%+ |

### 14.2 SSH Testing Approach

- **Unit tests**: mock russh primitives where possible
- **Integration tests**: spin up `linuxserver/openssh-server` Docker container in CI for real SSH handshake & I/O testing
- **Manual tests**: connect to real Linux/macOS/BSD servers before each release

### 14.3 Crypto Testing

- Roundtrip tests: encrypt → decrypt → equal to original
- Wrong-key tests: decryption with different key fails
- Tampered-ciphertext tests: GCM auth tag rejects tampered data
- Property-based tests via `proptest` for varied inputs

### 14.4 What NOT to Test

- Generated code (Tauri bindings)
- Third-party library internals
- Trivial getters/setters

---

## 15. Mobile Build & Distribution (Phase 10)

### 15.1 Tauri v2 Mobile Targets

```bash
# Initialize once
npm run tauri android init
npm run tauri ios init

# Dev runs
npm run tauri android dev
npm run tauri ios dev

# Production builds
npm run tauri android build --release
npm run tauri ios build --release
```

### 15.2 Android

| Aspect | Detail |
|--------|--------|
| Min SDK | 29 (Android 10) |
| Target SDK | latest at v1.0 release |
| Architectures | arm64-v8a (primary), armeabi-v7a, x86_64 |
| Build outputs | `.apk` (direct distribution), `.aab` (Play Store) |
| Signing | Local keystore for direct APK; Play App Signing managed for Play Store |
| Permissions declared | `INTERNET`, `USE_BIOMETRIC`, `POST_NOTIFICATIONS`, `WAKE_LOCK` (for keepalive) |

### 15.3 iOS

| Aspect | Detail |
|--------|--------|
| Min iOS | 15 |
| Architectures | arm64 |
| Distribution | TestFlight (beta), App Store (stable) |
| Signing | Apple Developer Program ($99/yr) — Distribution + Push (for biometric local) certificates |
| Capabilities | Background Modes (Audio: keep alive minimal time), Keychain Sharing, Face ID |

### 15.4 Mobile CI

GitHub Actions matrix needs macOS runners for iOS builds. Android builds can run on Linux. Cache:
- Android SDK + NDK
- iOS Pods
- Rust target dirs (`target/aarch64-linux-android`, `target/aarch64-apple-ios`, etc.)

### 15.5 Mobile Specific Tests

- Touch interaction tests (Playwright + mobile viewport simulation OR real device)
- Battery profiling (Android Profiler, Xcode Instruments)
- Background/foreground lifecycle (sessions persist briefly, auto-reconnect on resume)
- Biometric flow (manual smoke test per platform)

---

## 16. Plugin Distribution (Phase 12)

### 16.1 v1.0: Local Plugin Loading

For v1.0, plugins are loaded from local `.wasm` files. No public registry.

User workflow:
1. Download plugin file from author (GitHub Release, etc.)
2. App > Settings > Plugins > "Load from file..."
3. Review manifest (capabilities, signature, author)
4. Confirm install
5. Plugin runs sandboxed

### 16.2 Plugin Author Workflow

```bash
# Author writes plugin in Rust + targets wasm32-wasi
cargo build --target wasm32-wasi --release

# Sign manifest
shellmate-plugin-sign manifest.toml --key author.key

# Distribute the .wasm + manifest.toml + signature
```

### 16.3 Verification Pipeline

On install:
1. Verify manifest signature (Ed25519)
2. Display plugin metadata + capabilities to user
3. User clicks Approve → wrap with Wasmtime, instantiate
4. Plugin sandbox active

### 16.4 Public Registry (post-1.0)

Out of scope for v1.0. Future considerations:
- Centralized index of audited plugins
- Author identity verification
- Automated capability audit
- Update channel per plugin

---

## 17. Sync Backend Setup (Phase 9)

### 17.1 Per-Backend Adapter Requirements

| Backend | Auth | Setup Steps |
|---------|------|-------------|
| **iCloud** | macOS/iOS native | User signs in to iCloud at OS level; app uses CloudKit container |
| **GDrive** | OAuth 2.0 | App registered with Google API Console, user OAuth flow on first sync |
| **Dropbox** | OAuth 2.0 | App registered with Dropbox dev console |
| **S3 / MinIO** | Access key + secret | User provides bucket + credentials (encrypted at rest in vault) |
| **WebDAV** | Basic auth or token | URL + credentials |
| **Self-hosted HTTP** | API token | Custom endpoint + token |

### 17.2 Conflict Tests

Pre-release smoke test for each backend:
- Push from device A → pull on device B → matches
- Concurrent edit on A and B → conflict UI shown
- Revoke device → access cut off
- Backend outage → graceful retry with backoff

---

## 18. Security Audit Pipeline

### 18.1 Pre-1.0 Audit Checklist

- [ ] `cargo audit` clean (no known vulnerable dependencies)
- [ ] `npm audit` clean
- [ ] All secrets handling reviewed by 2nd pair of eyes
- [ ] Penetration test: SSH credential extraction, vault bypass, plugin sandbox escape
- [ ] Sync E2E test: verify cloud provider cannot read payloads (manual with provider tools)
- [ ] Biometric flow tested per platform
- [ ] CSP headers verified
- [ ] No analytics / telemetry calls (network monitor verification)

### 18.2 Continuous

- Dependabot enabled for npm + cargo
- Weekly `cargo audit` in CI
- Plugin SDK changes go through security review

---

*This document outlines the complete DevOps strategy and practices for ShellMate.*
