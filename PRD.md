# Product Requirements Document (PRD)
## SSH Client — Self-Hosted, Local-First, Multi-Device
**Codename:** ShellMate
**Version:** 2.0
**Author:** Matt
**Last Updated:** 2026-06-10
**Status:** In development (Phase 1-2 complete, scope expanded to full v1.0 production)

---

## 1. Overview

### 1.1 Latar Belakang

Saat ini banyak SSH client yang populer (seperti Termius) mengunci fitur-fitur esensial seperti host management, snippets, dan sync di balik paywall berlangganan. Alternatif gratis seperti PuTTY atau native terminal tidak menyediakan UX yang modern dan produktif, dan tidak ada satu pun yang menggabungkan **multi-device + local-first + extensibility** secara komprehensif.

ShellMate hadir sebagai SSH client yang **modern, lengkap, dan sepenuhnya lokal** — data tersimpan di device user sendiri, tidak ada server pihak ketiga, dan tidak ada biaya langganan. Ditujukan sebagai produk full v1.0 — bukan MVP — dengan dukungan desktop dan mobile, sinkronisasi end-to-end-encrypted via cloud milik user, plugin system, team vault, audit log, dan ekstensi protokol seperti Mosh.

### 1.2 Tujuan Produk

**Tujuan Utama:**

1. **Multi-SSH Connection** — Terhubung ke banyak server SSH secara bersamaan dalam satu aplikasi, masing-masing dalam tab/session independen, tanpa batas jumlah koneksi aktif. Termasuk **broadcast mode** untuk mengirim command ke beberapa session sekaligus.

2. **Multi-Device, Multi-Platform** — Tersedia di Windows, macOS, Linux, **Android**, dan **iOS**. UI adaptif untuk masing-masing form factor (keyboard-first di desktop, touch + extended key bar di mobile).

3. **Multi-Device Sync (E2E Encrypted)** — Sinkronisasi hosts, snippets, dan settings antar device secara opsional via user's own cloud (iCloud, GDrive, Dropbox, S3, WebDAV) atau self-hosted endpoint. Data dienkripsi di device sebelum upload — provider sync tidak bisa membaca isinya.

4. **Local-First & Privacy by Default** — Tidak ada server ShellMate yang terlibat. Semua state di device user. Telemetry zero. Vault terenkripsi dengan Argon2id + AES-256-GCM. Full-DB encryption via SQLCipher.

5. **Extensible** — Plugin system untuk fitur custom. Custom themes. Mosh support sebagai tambahan SSH.

**Tujuan Sekunder:**

- UX modern (clean, fast, keyboard-first di desktop; touch-friendly di mobile)
- Biometric unlock (Face ID, Touch ID, Windows Hello, Android Fingerprint)
- Team / sharing vault — share host config terenkripsi via team key
- Audit log untuk session start/end, file transfer, command history
- SFTP file browser dengan drag-and-drop
- Port forwarding (local & remote)
- Open-source (MIT)

### 1.3 Target Pengguna

| Segmen | Deskripsi |
|--------|-----------|
| **Developer** | Full-stack / backend dev yang SSH ke server staging/production dari desktop maupun HP |
| **DevOps / SysAdmin** | Manage banyak server sekaligus, butuh organisasi host, broadcast command, audit log |
| **Security Researcher** | Tool ringan untuk koneksi cepat ke banyak target dengan privacy-first design |
| **Power User** | User teknis yang ingin kontrol penuh atas data dan tools mereka |
| **Mobile User** | Dev/SysAdmin yang perlu SSH dari smartphone saat tidak di depan laptop |
| **Tim kecil** | 2-10 engineer yang ingin share host config tanpa pakai SaaS berbayar (Termius Premium dll) |

---

## 2. Scope

ShellMate v1.0 adalah **production release** — bukan MVP. Scope dibagi per area, bukan per phase, dan dikerjakan scope-driven (no fixed timeline).

### 2.1 Core Features (semua area in-scope untuk v1.0)

#### 2.1.1 Connection & Terminal
- Multi-tab SSH session (tanpa batas hard, soft warning di 20)
- 1 SSH connection per tab (per docs/04-backend §9)
- Password & SSH key auth (dengan passphrase)
- xterm.js terminal dengan ANSI colors, resize, copy/paste, search
- Terminal tab broadcast mode — kirim input yang sama ke beberapa tab
- SSH keepalive + auto-reconnect dengan exponential backoff
- Known hosts management dengan TOFU + warning untuk key mismatch
- **Mosh support** — fallback protocol untuk koneksi tidak stabil

#### 2.1.2 Host & Organization
- Host CRUD (add, edit, delete, validate)
- Host groups dengan nesting + drag-and-drop
- Tags, notes, search
- Import/export (encrypted JSON)

#### 2.1.3 Vault & Security
- Argon2id key derivation (memory-hard)
- AES-256-GCM untuk per-credential encryption
- **SQLCipher untuk full-DB encryption** (semua metadata terlindungi)
- Master password (length-first, 12-128 char per NIST SP 800-63B)
- No-recovery rule + onboarding warning + acknowledge gate
- Auto-lock after idle (configurable)
- Manual lock (Ctrl+L)
- Master password change with re-encryption
- **Biometric unlock**: Face ID, Touch ID, Windows Hello, Android Fingerprint
- Memory zeroize (Rust `zeroize`)

#### 2.1.4 Productivity
- Snippets (with template variables) + execution to active terminal
- Settings (theme, font, shortcuts, auto-lock, keepalive, scrollback)
- **Custom themes** — user-defined color schemes (terminal + UI)
- Keyboard shortcuts (configurable)

#### 2.1.5 File Transfer
- SFTP file browser (browse, upload, download, rename, delete, mkdir)
- Drag-and-drop upload
- Progress indicator
- Multiple SFTP windows per session
- SFTP runs as separate channel on the parent session's SSH connection

#### 2.1.6 Network
- Port forwarding (Local `-L` & Remote `-R`)
- Toggle rules without disconnecting
- Port conflict detection

#### 2.1.7 Multi-Device
- **Desktop**: Windows 10+, macOS 12+, Ubuntu 20.04+ (and equivalent Linux)
- **Mobile**: Android 10+, iOS 15+ via Tauri v2 mobile target
- Mobile UI: extended key bar (Esc, Tab, Ctrl, Alt, arrows, pipes), bottom-sheet navigation, full-screen SFTP modal, touch-friendly tab switcher

#### 2.1.8 Multi-Device Sync (E2E)
- Optional: device tetap fungsional tanpa sync
- User pilih backend: iCloud, GDrive, Dropbox, S3, WebDAV, atau self-hosted endpoint
- Encryption: payload dienkripsi di device sebelum upload (XChaCha20-Poly1305 atau AES-256-GCM dengan per-vault key)
- Conflict resolution: last-write-wins dengan timestamp + manual merge UI untuk konflik kompleks
- Selective sync: user pilih hosts/snippets mana yang di-sync
- Pause / disable kapan saja

#### 2.1.9 Team & Sharing
- **Team vault** — share host config terenkripsi via team key
- Member management (add member with public key, revoke, key rotation)
- Per-host share permissions (read-only / edit)
- Conflict resolution untuk shared host changes

#### 2.1.10 Plugin System
- Extension API: registered hooks pre/post connect, terminal data filter, custom UI panels
- WASM-based plugin runtime (sandboxed, no native code execution)
- Plugin permissions model (network, filesystem, secrets — semua opt-in)
- Plugin distribution: load from file, plugin manifest with signature

#### 2.1.11 Audit & Observability
- **Audit log** — session start/end, file transfer events, command history (opt-in per host)
- Log encrypted at rest, viewable via UI
- Export audit log (signed JSONL)
- Local error logging (no external services), log rotation

#### 2.1.12 Distribution & Updates
- Code signing: Windows Authenticode, macOS notarization, Linux GPG-signed AppImage
- **Auto-updater** via Tauri updater dengan signed releases
- Multi-arch builds: Windows x64, macOS Intel + Apple Silicon, Linux x64 + arm64

### 2.2 Out of Scope (v1.0)

- Cloud-hosted ShellMate service (always self-hosted / user's own cloud)
- Browser-based version
- Serial port / Telnet (security and modernity reasons)
- Container management (Docker, K8s) — pakai dedicated tool
- Built-in tmux replacement (use real tmux on the server)
- Telemetry/analytics
- Subscription tiers / paywalled features

### 2.3 Future Scope (post-1.0)

- Hardware key auth (FIDO2 / YubiKey)
- SSH agent forwarding (with explicit per-host opt-in)
- Cloud provider integration (AWS Session Manager, GCP IAP, Azure Bastion)
- Encrypted notes / runbooks per host
- Workflow automation (chain snippets across hosts)
- Public plugin registry

---

## 3. Tech Stack

### 3.1 Framework

| Layer | Teknologi | Alasan |
|-------|-----------|--------|
| App Framework | **Tauri v2** | Ringan (~10-20MB binary), pakai WebView OS bukan Chromium. Mobile target untuk Android & iOS. |
| Frontend | **React 18 + Vite + TypeScript (strict)** | Familiar, ekosistem besar, type-safe |
| Styling | **Tailwind CSS 3** | Utility-first, konsisten, theming via CSS vars |
| UI Components | **shadcn/ui** | Copy-paste components, accessible by default (Radix UI), customizable |
| Mobile UI Adaptation | **Responsive React + Tauri mobile APIs** | Touch handlers, extended key bar, bottom-sheet navigation |
| Terminal Emulator | **xterm.js** | Industry standard, feature-rich, search & WebGL addon |
| SSH Backend | **Rust (`russh` crate)** | Native performance, credentials tidak keluar dari Rust layer |
| Mosh Client | **Rust (custom mosh-client port atau wrapper)** | UDP SSP transport, fallback dari SSH |
| Local Storage | **SQLite via `rusqlite` + SQLCipher** | Full-DB encryption, single file |
| Per-credential Encryption | **AES-256-GCM** (defense in depth on top of SQLCipher) | Belt-and-suspenders untuk credential data |
| Key Derivation | **Argon2id** (memory-hard) | OWASP recommended |
| Sync Layer | **Custom encrypt-then-upload** dengan adapter per backend (iCloud, GDrive, S3, WebDAV) | E2E encryption, no ShellMate server |
| Plugin Runtime | **Wasmtime** (WASM, sandboxed) | Safe extension execution, capability-based permissions |
| Biometric | **OS-native APIs** via Tauri plugins (Touch ID, Face ID, Windows Hello, Android BiometricPrompt) | Native UX |
| State Management | **Zustand** | Simpel, ringan |
| Package Manager | **npm** (frontend), **Cargo** (backend) | Cross-platform Windows compatibility |

### 3.2 Arsitektur Umum

```
┌─────────────────────────────────────────┐
│           React UI (WebView)            │
│  xterm.js │ Host Manager │ SFTP Browser │
└──────────────────┬──────────────────────┘
                   │ invoke() / events
┌──────────────────▼──────────────────────┐
│            Rust Backend (Tauri)         │
│  SSH Handler │ SQLite │ Crypto Module   │
└──────────────────┬──────────────────────┘
                   │ TCP/SSH Protocol
┌──────────────────▼──────────────────────┐
│          Remote SSH Servers             │
│  Server A │ Server B │ Server C │ ...   │
└─────────────────────────────────────────┘
```

**Prinsip penting:** Credentials (password, private key) **hanya ada di Rust layer**. Frontend React tidak pernah memegang nilai credentials secara langsung — hanya berinteraksi via host ID.

---

## 4. Fitur Detail

### 4.1 Host Management

**Deskripsi:** User dapat menambahkan, mengedit, menghapus, dan mengorganisasi SSH hosts.

**Data per Host:**

| Field | Tipe | Wajib | Keterangan |
|-------|------|-------|-----------|
| `id` | UUID | Auto | Primary key |
| `label` | String | ✅ | Nama tampilan (e.g. "Production Web") |
| `hostname` | String | ✅ | IP atau domain |
| `port` | Integer | ✅ | Default: 22 |
| `username` | String | ✅ | SSH username |
| `auth_type` | Enum | ✅ | `password` / `key` / `key+passphrase` |
| `credential_ref` | UUID | ✅ | Referensi ke vault entry |
| `group_id` | UUID | ❌ | Referensi ke group |
| `tags` | String[] | ❌ | Label bebas (e.g. "prod", "aws") |
| `notes` | Text | ❌ | Catatan bebas |
| `created_at` | Timestamp | Auto | |
| `updated_at` | Timestamp | Auto | |

**Acceptance Criteria:**
- User dapat menambah host baru via form modal
- Form divalidasi sebelum disimpan (hostname tidak boleh kosong, port harus 1–65535)
- User dapat mengedit semua field host
- User dapat menghapus host dengan konfirmasi dialog
- Perubahan langsung tersimpan ke SQLite

---

### 4.2 Host Groups

**Deskripsi:** User dapat mengelompokkan hosts ke dalam grup untuk organisasi yang lebih baik.

**Data per Group:**

| Field | Tipe | Keterangan |
|-------|------|-----------|
| `id` | UUID | Primary key |
| `name` | String | Nama grup |
| `color` | HEX | Warna label (opsional) |
| `parent_id` | UUID | Untuk nested groups (opsional) |

**Acceptance Criteria:**
- Hosts dapat diorganisasi dalam grup (e.g. "Production", "Staging", "Dev")
- Grup bisa di-expand/collapse di sidebar
- Drag-and-drop host antar grup
- Grup bisa dihapus tanpa menghapus hosts di dalamnya (hosts menjadi ungrouped)

---

### 4.3 SSH Terminal

**Deskripsi:** Core feature — membuka terminal SSH interaktif ke host yang dipilih.

**Acceptance Criteria:**
- Klik connect pada host membuka tab terminal baru
- Terminal menggunakan xterm.js dengan full ANSI color support
- Support resize terminal (responsive terhadap ukuran window)
- Copy-paste berfungsi normal (Ctrl+C untuk interrupt, Ctrl+Shift+C untuk copy)
- Koneksi gagal menampilkan error message yang jelas
- Koneksi terputus otomatis menampilkan notifikasi + opsi reconnect
- Support SSH keepalive untuk mencegah timeout

**SSH Authentication Flow:**
```
User klik Connect
      ↓
Frontend invoke('ssh_connect', { host_id })
      ↓
Rust: ambil credentials dari vault (decrypt AES-256)
      ↓
Rust: buat SSH connection via russh
      ↓
Rust: buka channel, stream I/O via Tauri events
      ↓
Frontend: xterm.js render output
```

---

### 4.4 Multi-Tab Sessions (Multi-SSH Connection)

**Deskripsi:** User dapat terhubung ke **banyak server SSH sekaligus** dalam satu window, masing-masing dalam tab independen. Ini adalah **tujuan utama produk** dan **core feature v1.0** — ShellMate dirancang sejak awal untuk use case "connect ke banyak server secara bersamaan", bukan sekadar satu koneksi per window.

**Acceptance Criteria:**
- Setiap koneksi SSH berjalan dalam tab independen dengan session terpisah
- **Tidak ada batas hard** pada jumlah tab (limited by system resources)
- User dapat membuka koneksi ke **server yang sama** di beberapa tab sekaligus (multiple sessions per host)
- Nama tab menampilkan label host + username (e.g. `root@prod-web-01`)
- Tab color-coded berdasarkan group host (opsional, jika host punya warna group)
- Tab dapat di-close dengan konfirmasi jika session masih aktif
- Tab dapat di-reorder via drag-and-drop
- Shortcut keyboard untuk switch antar tab (Ctrl+Tab / Ctrl+1..9)
- Status indikator per tab: Connected 🟢 / Connecting 🟡 / Disconnected 🔴
- **Broadcast mode** *(v1.1)*: kirim input yang sama ke semua tab yang dipilih secara bersamaan (untuk batch command di banyak server)

---

### 4.5 Vault (Credential Storage)

**Deskripsi:** Penyimpanan credentials yang aman secara lokal.

**Skema Enkripsi:**
```
Master Password (dari user)
      ↓ PBKDF2 / Argon2
Derived Key (AES-256)
      ↓ Encrypt
Credentials (passwords, private keys) → tersimpan di SQLite
```

**Acceptance Criteria:**
- Credentials disimpan dalam bentuk terenkripsi AES-256-GCM di SQLite
- Vault dikunci dengan master password yang di-set saat pertama kali setup
- Master password tidak disimpan di mana pun — hanya digunakan untuk derive enkripsi key
- App dapat dikunci manual atau otomatis setelah idle X menit (user configurable)
- Biometric unlock (TouchID / Windows Hello) sebagai alternatif master password (v1.1)
- Import SSH private key dari file (.pem, .ppk, id_rsa, dll)
- Dukungan passphrase-protected private keys

---

### 4.6 Snippets

**Deskripsi:** Simpan command yang sering dipakai dan eksekusi dengan cepat.

**Data per Snippet:**

| Field | Tipe | Keterangan |
|-------|------|-----------|
| `id` | UUID | Primary key |
| `title` | String | Nama snippet |
| `command` | Text | Command string |
| `description` | Text | Deskripsi opsional |
| `tags` | String[] | Kategori |

**Acceptance Criteria:**
- User dapat membuat, edit, dan hapus snippet
- Snippet dapat dieksekusi ke terminal aktif dengan satu klik
- Snippet dapat dicari via search bar
- Snippet panel dapat dibuka via keyboard shortcut (Ctrl+K atau serupa)
- Support variabel dalam snippet (e.g. `ssh-copy-id {{username}}@{{host}}`)

---

### 4.7 Port Forwarding

**Deskripsi:** Konfigurasi dan manage SSH port forwarding rules.

**Tipe yang Didukung:**

| Tipe | Deskripsi | Contoh Use Case |
|------|-----------|----------------|
| **Local** | `-L local_port:remote_host:remote_port` | Akses DB remote via localhost |
| **Remote** | `-R remote_port:local_host:local_port` | Expose local service ke server |

**Acceptance Criteria:**
- User dapat mendefinisikan port forwarding rules per host
- Rules aktif ditampilkan dalam status panel
- Rules dapat di-toggle on/off tanpa disconnect SSH session
- Konflik port (port sudah dipakai) menampilkan error yang jelas

---

### 4.8 SFTP File Browser

**Deskripsi:** Browse dan transfer file ke/dari remote server secara visual.

**Acceptance Criteria:**
- SFTP browser membuka panel terpisah (bukan tab baru)
- Tampilkan directory listing dengan nama, size, permission, dan modified date
- Upload file via drag-and-drop atau file picker
- Download file dengan klik kanan → Download
- Operasi dasar: rename, delete, buat folder baru
- Progress indicator untuk transfer file besar
- SFTP berjalan dalam koneksi SSH yang sama (tidak buka koneksi baru)

---

### 4.9 Mobile Support (Phase 10)

**Deskripsi:** ShellMate tersedia di smartphone (Android & iOS) via Tauri v2 mobile target, memungkinkan user SSH dari HP kapan saja.

**Perbedaan Mobile vs Desktop:**

| Aspek | Desktop | Mobile |
|-------|---------|--------|
| Input | Keyboard fisik | Virtual keyboard + extended key bar |
| Navigasi tab | Tab bar horizontal | Bottom sheet / swipe navigation |
| SFTP | Panel terpisah | Full-screen modal |
| Shortcuts | Keyboard shortcuts | Gesture-based (swipe, long-press) |
| Font size | 14px default | 12-16px adjustable |
| Orientation | Landscape preferred | Portrait + Landscape |

**Extended Key Bar (Mobile):**
Baris tombol tambahan di atas virtual keyboard untuk karakter yang sering dipakai di terminal:

```
[Esc] [Tab] [Ctrl] [Alt] [↑] [↓] [←] [→] [|] [~] [-] [/]
```

**Acceptance Criteria (Mobile, Phase 10):**
- Semua fitur core SSH terminal berjalan di Android & iOS
- **Multi-tab tetap tersedia** di mobile — user dapat switch antar server via swipe atau tab navigator
- Extended key bar selalu tampil di atas virtual keyboard
- Font size dapat diatur via pinch-to-zoom atau settings
- Auto-rotate support (portrait & landscape)
- Biometric unlock (Face ID, Fingerprint) sebagai alternatif master password
- Notifikasi saat koneksi SSH terputus (background disconnect alert)

---

### 4.10 Multi-Device Sync (Phase 9)

**Deskripsi:** Sinkronisasi konfigurasi (hosts, groups, snippets) antara desktop dan mobile secara opsional. Data selalu terenkripsi sebelum keluar dari device.

**Model Sync:**

```
Device A (Desktop)                Device B (HP)
      │                                 │
      ▼                                 ▼
  Encrypt (AES-256)             Encrypt (AES-256)
      │                                 │
      └──────────► Sync Storage ◄───────┘
                  (iCloud / GDrive /
                   S3 / Self-hosted)
```

**Acceptance Criteria:**
- Sync sepenuhnya **opsional** — app tetap fully functional tanpa sync
- User pilih sendiri storage backend (iCloud, GDrive, Dropbox, S3, WebDAV)
- Data dienkripsi di device sebelum upload — provider sync tidak bisa membaca isinya
- Conflict resolution: last-write-wins dengan timestamp, atau manual merge UI
- Sync dapat di-pause atau di-disable kapan saja
- Tidak ada server ShellMate yang terlibat — pure peer-to-peer via user's own storage



### 5.1 Layout Umum

```
┌──────────────┬────────────────────────────────────┐
│   Sidebar    │         Main Content Area          │
│              │                                    │
│  [Search]    │  ┌──┬──┬──┬──────────────────┐    │
│              │  │T1│T2│T3│         +        │    │
│  ▼ Production│  └──┴──┴──┴──────────────────┘    │
│    Web-01    │                                    │
│    DB-01     │   [Terminal / SFTP / Settings]     │
│  ▼ Staging   │                                    │
│    Web-01    │                                    │
│  + Add Host  │                                    │
│              │                                    │
│  [Snippets]  │                                    │
│  [Settings]  │                                    │
└──────────────┴────────────────────────────────────┘
```

### 5.2 Design Principles

- **Dark mode by default** (bisa toggle ke light)
- **Minimalis** — tidak ada elemen UI yang tidak perlu
- **Keyboard-first** — semua aksi utama dapat dilakukan via keyboard
- **Konsisten** — gunakan design tokens yang konsisten (spacing, warna, typography)
- Font terminal: **JetBrains Mono** atau **Fira Code** (dapat dikonfigurasi user)

### 5.3 Keyboard Shortcuts

| Shortcut | Aksi |
|----------|------|
| `Ctrl+T` | Buka tab baru (dari host terakhir) |
| `Ctrl+W` | Tutup tab aktif |
| `Ctrl+Tab` | Switch ke tab berikutnya |
| `Ctrl+Shift+Tab` | Switch ke tab sebelumnya |
| `Ctrl+K` | Buka snippet panel |
| `Ctrl+Shift+F` | Buka SFTP browser |
| `Ctrl+,` | Buka Settings |
| `Ctrl+L` | Lock vault |
| `Ctrl+F` | Search di terminal (xterm.js search addon) |

---

## 6. Settings & Konfigurasi

| Setting | Default | Keterangan |
|---------|---------|-----------|
| Theme | Dark | Dark / Light / System |
| Font family | JetBrains Mono | Terminal font |
| Font size | 14px | Terminal font size |
| Scrollback lines | 5000 | Buffer baris terminal |
| Auto-lock timeout | 15 menit | 0 = disabled |
| SSH keepalive interval | 60 detik | Kirim keepalive packet |
| SSH keepalive max retries | 3 | Sebelum disconnect |
| Default SSH port | 22 | |
| Confirm on tab close | true | |
| Cursor style | Block | Block / Bar / Underline |
| Cursor blink | true | |

---

## 7. Data Storage

### 7.1 Lokasi File

| OS | Path |
|----|------|
| Windows | `%APPDATA%\ShellMate\` |
| macOS | `~/Library/Application Support/ShellMate/` |
| Linux | `~/.config/shellmate/` |

### 7.2 Database Schema (SQLite)

```sql
-- Hosts
CREATE TABLE hosts (
  id TEXT PRIMARY KEY,
  label TEXT NOT NULL,
  hostname TEXT NOT NULL,
  port INTEGER NOT NULL DEFAULT 22,
  username TEXT NOT NULL,
  auth_type TEXT NOT NULL CHECK (auth_type IN ('password', 'key', 'key_passphrase')),
  credential_id TEXT NOT NULL REFERENCES credentials(id),
  group_id TEXT REFERENCES groups(id),
  tags TEXT, -- JSON array
  notes TEXT,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

-- Groups
CREATE TABLE groups (
  id TEXT PRIMARY KEY,
  name TEXT NOT NULL,
  color TEXT,
  parent_id TEXT REFERENCES groups(id),
  sort_order INTEGER DEFAULT 0
);

-- Credentials (encrypted)
CREATE TABLE credentials (
  id TEXT PRIMARY KEY,
  type TEXT NOT NULL CHECK (type IN ('password', 'private_key')),
  encrypted_data BLOB NOT NULL, -- AES-256-GCM encrypted
  nonce BLOB NOT NULL,          -- GCM nonce
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

-- Snippets
CREATE TABLE snippets (
  id TEXT PRIMARY KEY,
  title TEXT NOT NULL,
  command TEXT NOT NULL,
  description TEXT,
  tags TEXT, -- JSON array
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

-- Port forwarding rules
CREATE TABLE port_forwards (
  id TEXT PRIMARY KEY,
  host_id TEXT NOT NULL REFERENCES hosts(id),
  type TEXT NOT NULL CHECK (type IN ('local', 'remote')),
  local_port INTEGER NOT NULL,
  remote_host TEXT NOT NULL,
  remote_port INTEGER NOT NULL,
  enabled INTEGER NOT NULL DEFAULT 1
);

-- App settings
CREATE TABLE settings (
  key TEXT PRIMARY KEY,
  value TEXT NOT NULL
);
```

---

## 8. Security Requirements

| Requirement | Detail |
|-------------|--------|
| Credentials at rest | AES-256-GCM terenkripsi sebelum masuk SQLite |
| Master key derivation | Argon2id (memory-hard, brute-force resistant) |
| Memory safety | Credentials di-zeroize dari memory setelah digunakan (Rust `zeroize` crate) |
| No plaintext logging | Credentials tidak pernah muncul di log, console, atau error message |
| SSH host verification | Known hosts disimpan lokal, warning untuk host baru atau key mismatch |
| No telemetry | Tidak ada data usage yang dikirim ke mana pun |
| Vault lock | Auto-lock setelah idle, manual lock via shortcut |

---

## 9. Non-Functional Requirements

| Aspek | Target |
|-------|--------|
| **Startup time** | < 2 detik cold start |
| **Memory usage** | < 50MB idle, < 100MB dengan 5 tab aktif |
| **Binary size** | < 20MB installer |
| **SSH latency** | Overhead < 5ms dari native SSH client |
| **Supported OS (Desktop)** | Windows 10+, macOS 12+, Ubuntu 20.04+ |
| **Supported OS (Mobile)** | Android 10+, iOS 15+ *(Phase 10)* |
| **Offline** | Fully functional tanpa internet (kecuali koneksi ke server target) |

---

## 10. Milestones & Roadmap

ShellMate v1.0 dikerjakan **scope-driven** (no fixed timeline). Tiap milestone ship saat acceptance criteria terpenuhi. Milestone diberi target ordering, bukan deadline.

### Phase 1 — Project Setup ✅ (2026-06-09)
- [x] Tauri v2 + React/Vite/TS scaffold
- [x] Tailwind CSS 3 + custom dark theme
- [x] SQLite schema + migrations
- [x] Layout shell (sidebar, tab bar, status bar, custom title bar)
- [x] Zustand stores (host, tab, ui)
- [x] ESLint + Prettier + tsconfig strict
- [x] MIT LICENSE + CHANGELOG
- [x] CI ready (typecheck, lint, build all green)

### Phase 2 — Core SSH ✅ (2026-06-10)
- [x] Vault: Argon2id + AES-256-GCM + zeroize
- [x] Vault setup/unlock/lock + recovery warning UI
- [x] SSH connection via password & key auth (russh)
- [x] xterm.js terminal integration
- [x] Multi-tab session manager (1 connection per tab)
- [x] QuickConnect form for testing

### Phase 3 — Host Management & Persistence ✅ (2026-06-10)
- [x] Host CRUD UI (form, list, edit, delete)
- [x] Groups + drag-and-drop
- [x] Tags, notes, host search
- [x] Save credentials via vault, connect from sidebar
- [x] Host validation (frontend + backend sync)

### Phase 4 — Productivity & Settings ✅ (2026-06-10)
- [x] Snippets panel (CRUD, search, execute to terminal, template variables)
- [x] Settings dialog (theme, font, shortcuts, keepalive, scrollback)
- [x] Custom themes — 3 built-ins via CSS variables; storage backend ready for custom themes
- [ ] Configurable keyboard shortcuts — deferred to Phase 14 polish (defaults work)
- [x] Auto-lock UX wired (frontend polls `vault_check_idle`)
- [x] Master password change with full re-encryption

### Phase 5 — File Transfer & Network ✅ (2026-06-10)
- [x] SFTP file browser (browse, upload, download, rename, delete, mkdir)
- [x] SFTP drag-and-drop, progress indicator
- [x] Port forwarding (local & remote, toggle, conflict detection)

### Phase 6 — Network Hardening ✅ (2026-06-10)
- [x] Known hosts table + verification UI (TOFU with fingerprint display)
- [x] Auto-reconnect with exponential backoff (1s→60s)
- [x] Broadcast mode (kirim command ke beberapa session)
- [ ] Mosh client integration (UDP SSP transport) — **deferred to Phase 14**

### Phase 7 — Full-DB Encryption
- [ ] Migrate from per-field encryption only → SQLCipher full-DB encryption (defense in depth, both layers active)
- [ ] Migration tool (one-shot for existing databases)
- [ ] Performance benchmark before/after

### Phase 8 — Biometric Unlock
- [ ] Tauri plugin integration: Touch ID (macOS), Windows Hello, Face ID/fingerprint (mobile)
- [ ] Vault key wrapped with biometric-protected secure enclave key
- [ ] Fallback ke master password kalau biometric gagal/disabled

### Phase 9 — Multi-Device Sync (E2E)
- [ ] Sync engine architecture (encrypt-then-upload, manifest, conflict resolution)
- [ ] Backend adapters: iCloud, GDrive, Dropbox, S3, WebDAV
- [ ] Selective sync UI (per host/snippet/group)
- [ ] Conflict merge UI
- [ ] Sync log + diagnostic

### Phase 10 — Mobile (Android & iOS)
- [ ] Tauri v2 mobile target setup (Android first, then iOS)
- [ ] Adaptive UI: bottom-sheet nav, full-screen panels
- [ ] Extended key bar (Esc, Tab, Ctrl, Alt, arrows, pipe, tilde, slash)
- [ ] Touch-friendly host list, tab switcher (swipe), SFTP modal
- [ ] Mobile-specific shortcuts via gestures
- [ ] Background reconnect handling
- [ ] Notification on session disconnect

### Phase 11 — Team Vault
- [ ] Team member management (add via public key, revoke, key rotation)
- [ ] Per-host share permissions (read-only / edit)
- [ ] Encrypted host export with team key wrap
- [ ] Conflict resolution for shared host changes

### Phase 12 — Plugin System
- [ ] Wasmtime runtime integration
- [ ] Plugin API: hooks (pre/post connect, terminal data filter), custom UI panels
- [ ] Capability-based permissions (network, fs, secrets — all opt-in per plugin)
- [ ] Plugin manifest + signing
- [ ] Plugin distribution: load from file, sample plugins shipped

### Phase 13 — Audit Log
- [ ] Audit event capture (session start/end, SFTP transfers, command history if opt-in per host)
- [ ] Encrypted audit log storage
- [ ] Audit log viewer UI (filter, search, export signed JSONL)
- [ ] Privacy: opt-in per host, redaction rules

### Phase 14 — Polish & Distribution
- [ ] Onboarding flow (first-launch tutorial, vault setup walkthrough)
- [ ] Error handling + reconnect UX
- [ ] Export/import hosts (encrypted JSON) — fallback for sync-disabled users
- [ ] Performance audit (bundle size, startup, memory)
- [ ] Full a11y pass (axe-core CI, manual NVDA + VoiceOver)
- [ ] Cross-platform testing
- [ ] Code signing setup: Windows Authenticode, macOS notarization, Linux GPG
- [ ] Tauri auto-updater with signed releases
- [ ] App packaging: Windows .msi, macOS .dmg, Linux .AppImage + .deb, Android .apk/.aab, iOS via TestFlight then App Store
- [ ] User documentation (install, getting started, features, troubleshooting)
- [ ] Release v1.0.0

---

## 11. Resolved Decisions

| # | Topik | Keputusan | Alasan |
|---|-------|-----------|--------|
| 1 | Nama final | **ShellMate** | Distinctive, sudah dipakai konsisten di docs |
| 2 | Mosh support | **In v1.0** (Phase 6) | User-driven scope expansion. Akan ditambahkan sebagai protocol fallback. |
| 3 | SFTP scope | **In v1.0** (Phase 5) | Core productivity feature |
| 4 | License | **MIT** | Permissive, kompatibel dengan tujuan open-source self-hostable |
| 5 | Auto-updater | **In v1.0** (Phase 14) | Production app butuh delivery channel. Tauri v2 updater dengan signing. |
| 6 | Mobile platforms | **Android + iOS, Android first** | Android dev cycle lebih cepat (no Apple Developer cert untuk dev), iOS menyusul |
| 7 | Multi-device sync arsitektur | **User's own cloud, no ShellMate server** | Privacy-first, sesuai prinsip local-first. E2E encryption sebelum upload. |
| 8 | Broadcast mode | **In v1.0** (Phase 6) | Core productivity feature for sysadmin |
| 9 | Encryption granularity | **Per-credential AES-256-GCM + SQLCipher full-DB encryption** (defense in depth) | Per-field melindungi credentials, SQLCipher melindungi metadata. Both layers active in v1.0. |
| 10 | Master password recovery | **No recovery** (lupa password = data hilang) | Standar untuk vault local-first. UX explicit di onboarding. |
| 11 | Master password policy | **Length-first** (min 12 karakter), tidak wajib karakter khusus | NIST SP 800-63B (2017+) rekomen length over complexity |
| 12 | Connection sharing multi-tab | **1 SSH connection per tab** untuk v1.0 | Isolation lebih baik, lebih simpel. Multiplex evaluasi post-1.0. |
| 13 | Plugin system | **In v1.0** (Phase 12) — Wasmtime sandbox | Capability-based permissions, no native code execution. Safety-first extensibility. |
| 14 | Team / sharing vault | **In v1.0** (Phase 11) | Differentiator vs Termius (yang paywall team feature). |
| 15 | Audit log | **In v1.0** (Phase 13), opt-in per host | Privacy default off, useful for compliance / DevOps. |
| 16 | Custom themes | **In v1.0** (Phase 4) | UX expectation untuk modern terminal app. |
| 17 | Biometric unlock | **In v1.0** (Phase 8) — desktop + mobile | UX critical untuk frequent unlock. Tauri plugin per OS. |
| 18 | Mobile mode terminal protocol | **SSH first, Mosh as enhancement** | Mosh sangat valuable di mobile (network changes), tapi SSH dulu. |
| 19 | Sync conflict strategy | **Last-write-wins + manual merge UI** untuk konflik kompleks | Sederhana untuk umum, fleksibel untuk power user. |
| 20 | Versioning approach | **Scope-driven, no fixed timeline** | Phase ship saat acceptance terpenuhi. Quality > deadline. |

---

## 12. Referensi

- [Tauri v2 Documentation](https://v2.tauri.app/)
- [russh crate](https://crates.io/crates/russh)
- [xterm.js](https://xtermjs.org/)
- [rusqlite](https://crates.io/crates/rusqlite)
- [Termius](https://termius.com) — referensi UX
