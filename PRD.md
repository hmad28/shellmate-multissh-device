# Product Requirements Document (PRD)
## SSH Client Desktop App — Self-Hosted, Local-First
**Codename:** ShellMate *(working title, bisa diganti)*
**Version:** 1.1
**Author:** Matt
**Last Updated:** 2026-06-03
**Status:** Draft

---

## 1. Overview

### 1.1 Latar Belakang

Saat ini banyak SSH client yang populer (seperti Termius) mengunci fitur-fitur esensial seperti host management, snippets, dan sync di balik paywall berlangganan. Alternatif gratis seperti PuTTY atau native terminal tidak menyediakan UX yang modern dan produktif.

ShellMate hadir sebagai SSH client yang **ringan, modern, dan sepenuhnya lokal** — data tersimpan di device user sendiri, tidak ada server pihak ketiga, dan tidak ada biaya langganan. ShellMate dirancang untuk mendukung **koneksi ke banyak server SSH secara bersamaan** dan dapat diakses dari **berbagai device**, termasuk desktop, laptop, dan smartphone (Android & iOS).

### 1.2 Tujuan Produk

**Tujuan Utama:**

1. **Multi-SSH Connection** — Memungkinkan user untuk terhubung ke banyak server SSH secara bersamaan dalam satu aplikasi, masing-masing dalam tab/session independen, tanpa batas jumlah koneksi aktif.

2. **Multi-Device Support** — ShellMate tersedia di semua platform utama: desktop (Windows, macOS, Linux) **dan** mobile (Android & iOS), dengan data dan konfigurasi yang dapat disinkronkan antar device secara opsional.

**Tujuan Lainnya:**

- Menyediakan SSH client modern dengan UX yang bersih dan cepat
- Menyimpan semua data (hosts, credentials, snippets) secara lokal di device user
- Mendukung koneksi ke banyak server sekaligus via multi-tab
- Ringan di sumber daya sistem (CPU, RAM, disk)
- Open-source friendly dan self-hostable

### 1.3 Target Pengguna

| Segmen | Deskripsi |
|--------|-----------|
| **Developer** | Full-stack / backend dev yang sering SSH ke server staging/production dari desktop maupun HP |
| **DevOps / SysAdmin** | Manage banyak server sekaligus, butuh organisasi host yang baik dan akses dari mana saja |
| **Security Researcher** | Butuh tool ringan untuk koneksi cepat ke banyak target |
| **Power User** | User teknis yang ingin kontrol penuh atas data dan tools mereka |
| **Mobile User** | Developer/SysAdmin yang perlu SSH dari smartphone saat tidak di depan laptop |

---

## 2. Scope

### 2.1 In Scope (MVP — Desktop)

- Manajemen hosts (add, edit, delete, group)
- SSH terminal via WebView (xterm.js)
- **Multi-tab session** — koneksi ke banyak server SSH sekaligus
- Autentikasi via password dan SSH key
- Penyimpanan lokal terenkripsi (SQLite + AES-256)
- Snippets / command shortcuts
- Port forwarding (Local & Remote)
- SFTP file browser dasar
- Platform: Windows, macOS, Linux

### 2.2 Out of Scope (MVP)

- Cloud sync / remote backup
- Serial/Telnet connection
- Terminal multiplexer (tmux-like)
- Container management (Docker, K8s)
- Plugin system

### 2.3 Future Scope (Post-MVP)

- **Mobile App (Android & iOS)** — Tauri v2 mobile target; UI adaptif untuk layar kecil dan touch input, termasuk virtual keyboard dengan tombol khusus (Tab, Ctrl, Esc, arrow keys)
- **Multi-Device Sync** — Sinkronisasi hosts, snippets, dan settings secara opsional via user's own cloud (iCloud, GDrive, S3) atau self-hosted endpoint; data tetap terenkripsi end-to-end
- Export/import hosts (JSON terenkripsi)
- Team/sharing vault
- Audit log
- Biometric unlock di mobile (Face ID, Fingerprint)

---

## 3. Tech Stack

### 3.1 Framework

| Layer | Teknologi | Alasan |
|-------|-----------|--------|
| App Framework | **Tauri v2** | Ringan (~5-10MB binary), pakai WebView OS bukan Chromium |
| Frontend | **React + Vite** | Familiar, ekosistem besar, cepat |
| Styling | **Tailwind CSS** | Utility-first, konsisten |
| Terminal Emulator | **xterm.js** | Industry standard, feature-rich |
| SSH Backend | **Rust (`russh` crate)** | Native performance, credentials tidak keluar dari Rust layer |
| Local Storage | **SQLite via `rusqlite`** | Ringan, reliable, satu file database |
| Enkripsi | **AES-256-GCM** | Enkripsi credentials sebelum disimpan ke SQLite |
| State Management | **Zustand** | Simpel, ringan |

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

**Deskripsi:** User dapat terhubung ke **banyak server SSH sekaligus** dalam satu window, masing-masing dalam tab independen. Ini adalah **tujuan utama produk** dan **core feature MVP** — ShellMate dirancang sejak awal untuk use case "connect ke banyak server secara bersamaan", bukan sekadar satu koneksi per window.

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

### 4.9 Mobile Support (Post-MVP)

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

**Acceptance Criteria (Mobile MVP):**
- Semua fitur core SSH terminal berjalan di Android & iOS
- **Multi-tab tetap tersedia** di mobile — user dapat switch antar server via swipe atau tab navigator
- Extended key bar selalu tampil di atas virtual keyboard
- Font size dapat diatur via pinch-to-zoom atau settings
- Auto-rotate support (portrait & landscape)
- Biometric unlock (Face ID, Fingerprint) sebagai alternatif master password
- Notifikasi saat koneksi SSH terputus (background disconnect alert)

---

### 4.10 Multi-Device Sync (Post-MVP)

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
| **Supported OS (Mobile)** | Android 10+, iOS 15+ *(Post-MVP)* |
| **Offline** | Fully functional tanpa internet (kecuali koneksi ke server target) |

---

## 10. Milestones & Roadmap

### Milestone 1 — Core SSH (Target: 2 minggu)
- [ ] Scaffold Tauri v2 + React + Vite project
- [ ] SQLite setup + schema migration
- [ ] CRUD hosts (add, edit, delete)
- [ ] Vault + enkripsi AES-256
- [ ] SSH connect via password auth
- [ ] Terminal xterm.js rendering
- [ ] **Multi-tab session** *(core use case: connect ke banyak server sekaligus)*
- [ ] Basic sidebar layout

### Milestone 2 — Productionize (Target: +1 minggu)
- [ ] SSH key authentication
- [ ] Groups & tags
- [ ] Snippets panel
- [ ] Keyboard shortcuts
- [ ] Settings page

### Milestone 3 — Power Features (Target: +2 minggu)
- [ ] Port forwarding
- [ ] SFTP file browser
- [ ] Search hosts
- [ ] Terminal search (xterm.js addon)
- [ ] Known hosts management
- [ ] Auto-lock vault

### Milestone 4 — Polish Desktop (Target: +1 minggu)
- [ ] Dark/light theme
- [ ] Onboarding flow (first launch)
- [ ] Error handling & reconnect UX
- [ ] Export/import hosts (JSON terenkripsi)
- [ ] App packaging & installer (Windows .msi, macOS .dmg, Linux .AppImage)

### Milestone 5 — Mobile App Android & iOS (Target: Post-MVP)
- [ ] Tauri v2 mobile target setup (Android & iOS)
- [ ] Adaptive UI untuk layar kecil
- [ ] Extended key bar (Esc, Tab, Ctrl, arrow keys)
- [ ] Multi-tab navigation di mobile (swipe / bottom navigator)
- [ ] Biometric unlock (Face ID, Fingerprint)
- [ ] Touch-friendly SFTP browser

### Milestone 6 — Multi-Device Sync (Target: Post-MVP)
- [ ] Sync engine (encrypted export/import)
- [ ] iCloud & GDrive backend
- [ ] Self-hosted / WebDAV backend
- [ ] Conflict resolution UI

---

## 11. Open Questions

| # | Pertanyaan | Status |
|---|-----------|--------|
| 1 | Nama final app? "ShellMate" atau nama lain? | ❓ Open |
| 2 | Apakah perlu support Mosh di MVP? | ❓ Open |
| 3 | Apakah SFTP bisa ditunda ke post-MVP? | ❓ Open |
| 4 | Lisensi: MIT / Apache 2.0 / proprietary? | ❓ Open |
| 5 | Apakah perlu auto-updater di MVP? | ❓ Open |
| 6 | Mobile: apakah Android atau iOS yang dikerjakan lebih dulu? | ❓ Open |
| 7 | Multi-device sync: apakah pakai self-hosted server ShellMate atau murni user's own cloud? | ❓ Open |
| 8 | Broadcast mode (kirim command ke banyak server sekaligus) — masuk MVP atau post-MVP? | ❓ Open |

---

## 12. Referensi

- [Tauri v2 Documentation](https://v2.tauri.app/)
- [russh crate](https://crates.io/crates/russh)
- [xterm.js](https://xtermjs.org/)
- [rusqlite](https://crates.io/crates/rusqlite)
- [Termius](https://termius.com) — referensi UX
