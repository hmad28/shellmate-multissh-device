# Shellmate: Cross-Platform Architecture & P2P Sync Plan

This document outlines the architecture and implementation plan for transforming Shellmate from a standalone desktop app into a cross-platform (Desktop + Mobile) ecosystem with peer-to-peer (P2P) synchronization.

## 1. Ecosystem Overview

Shellmate will consist of two primary clients sharing the same core Rust (`russh`) engine:
- **Shellmate Desktop (.exe)**: The master client running on the user's laptop.
- **Shellmate Mobile (.apk)**: The portable client running on Android.

Both clients will independently store their own SQLite databases and encrypted Vaults. They can connect to external servers (VPS, Raspberry Pi, etc.) autonomously.

## 2. P2P Local Sync Architecture

To achieve a true "Local-First" Termius alternative, Shellmate will bypass cloud servers entirely and use a secure P2P sync mechanism.

### Discovery
- Devices will discover each other over the local network or VPN (e.g., Tailscale) using **mDNS** (`mdns-sd`).
- The Desktop app will broadcast a `_shellmate_sync._tcp.local.` service.

### Secure Pairing & Transfer
1. **Initiation**: The user clicks "Sync Devices" on the Mobile app, which detects the Desktop app.
2. **Handshake**: A temporary, secure WebSocket/HTTP server spins up on the Desktop app.
3. **Authentication**: The Desktop UI displays a temporary 6-digit PIN or QR code. The Mobile app scans/enters it to authenticate the session.
4. **Data Exchange**: The Desktop app encrypts its SQLite database and Vault blob using a session key derived from the PIN, and transmits it to the Mobile app.
5. **Merge**: The Mobile app merges the imported hosts and credentials into its local database.

## 3. "VIP" Passwordless Admin Access

To fulfill the requirement of seamlessly connecting to the laptop's admin terminal from the phone:
1. **Key Generation**: Shellmate Desktop will automatically generate a dedicated `ed25519` SSH keypair for "Mobile Access".
2. **Injection**: Shellmate Desktop will inject the public key into the Windows OpenSSH `authorized_keys` (or `administrators_authorized_keys` if running as admin).
3. **Syncing**: The corresponding private key is securely synced to the Mobile app during the P2P Sync process.
4. **Result**: The Mobile app receives a pre-configured Host entry for the Laptop. Tapping it establishes an instant, passwordless SSH connection to the Windows terminal.

## 4. Implementation Phases

### Phase 1: Android Build Pipeline (WIP)
- [x] Initialize Tauri Android project.
- [x] Configure Android networking/mDNS permissions.
- [x] Set up mobile-responsive UI adjustments (Tailwind).
- [ ] Successfully compile and test the APK on a physical device/emulator.

### Phase 2: VIP Access Provisioning ✅ (2026-06-10)
- [x] Add Rust backend command to generate `ed25519` keypair (`vip_access.rs`).
- [x] Add Rust backend command to append to Windows OpenSSH `authorized_keys`.
- [x] Create a specific Host entry in the database pointing to `localhost`.
- [x] Frontend UI panel for VIP access configuration (`VipAccessPanel.tsx`).
- [x] Tauri commands: `vip_generate_keypair`, `vip_inject_authorized_keys`, `vip_create_localhost_host`, `vip_get_key_status`.

### Phase 3: P2P Sync Engine ✅ (2026-06-10)
- [x] Implement a lightweight HTTP server in the Rust backend (`p2p_sync.rs`) with `tokio::net::TcpListener`.
- [x] Implement secure PIN-based payload encryption (AES-256-GCM with SHA-256 derived key).
- [x] Build the UI flow for Pairing (Display PIN on Desktop, Input PIN on Mobile) (`P2pSyncPanel.tsx`).
- [x] Implement the database merge logic on the receiving end (deduplicates by ID, imports groups → credentials → hosts → snippets).
- [x] Tauri commands: `p2p_start_sync_server`, `p2p_stop_sync_server`, `p2p_get_sync_status`, `p2p_export_for_sync`.

### Implementation Notes (2026-06-10)

**VIP Access:**
- Uses `ed25519-dalek` crate for keypair generation.
- Private key encrypted with vault and stored as credential in the `credentials` table.
- Public key appended to `~/.ssh/authorized_keys` with `shellmate-vip` comment.
- Host entry created with `hostname = 'localhost'` (resolves to laptop's IP over Tailscale for remote access).

**P2P Sync:**
- HTTP server binds to `0.0.0.0:0` (random port) for local network access.
- PIN is 6-digit numeric (1M combinations), displayed on desktop for mobile to enter.
- `GET /pair` endpoint returns PIN for mobile pairing.
- `POST /sync` endpoint receives encrypted payload, decrypts, and merges into local DB.
- Encryption: AES-256-GCM with key derived from SHA-256 of `"shellmate-sync-v1" + PIN`.
- `p2p_export_for_sync` exports all data (hosts, credentials, groups, snippets) as base64-encoded JSON.
- Server shuts down after sync completes or when manually stopped.

---

*This plan ensures a fully local, privacy-respecting, and highly convenient experience without relying on any third-party cloud infrastructure.*
