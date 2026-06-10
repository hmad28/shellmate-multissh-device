# Security Plan
## ShellMate — Security Architecture (v1.0 Production)

**Version:** 2.0
**Last Updated:** 2026-06-10

---

## 1. Security Overview

### 1.1 Security Principles
1. **Defense in Depth** - Multiple layers of security
2. **Least Privilege** - Minimal access required
3. **Fail Secure** - Secure by default
4. **Zero Trust** - Verify everything
5. **Privacy by Design** - Data protection built-in

### 1.2 Threat Model
| Threat | Impact | Mitigation |
|--------|--------|------------|
| Credential theft | Critical | AES-256-GCM encryption, zeroize |
| Memory dump | Critical | Zeroize sensitive data |
| SQL injection | High | Parameterized queries |
| Man-in-the-middle | High | SSH host verification |
| Brute force | High | Argon2id key derivation |
| Unauthorized access | High | Vault lock, auto-lock |
| Data exfiltration | High | No telemetry, local-only |

---

## 2. Encryption Architecture

### 2.1 Encryption Layers
```
┌─────────────────────────────────────────────────────────┐
│                    Encryption Layers                     │
├─────────────────────────────────────────────────────────┤
│                                                          │
│  Layer 1: Key Derivation                                │
│  ┌─────────────────────────────────────────────────┐   │
│  │  Master Password (user input)                    │   │
│  │         ↓                                        │   │
│  │  Argon2id (memory-hard KDF)                      │   │
│  │         ↓                                        │   │
│  │  Derived Key (256-bit)                           │   │
│  └─────────────────────────────────────────────────┘   │
│                                                          │
│  Layer 2: Data Encryption                               │
│  ┌─────────────────────────────────────────────────┐   │
│  │  Plaintext Data                                  │   │
│  │         ↓                                        │   │
│  │  AES-256-GCM (authenticated encryption)          │   │
│  │         ↓                                        │   │
│  │  Ciphertext + Nonce + Auth Tag                   │   │
│  └─────────────────────────────────────────────────┘   │
│                                                          │
│  Layer 3: Transport Encryption                          │
│  ┌─────────────────────────────────────────────────┐   │
│  │  SSH Protocol (RFC 4253)                         │   │
│  │         ↓                                        │   │
│  │  Encrypted TCP Connection                        │   │
│  └─────────────────────────────────────────────────┘   │
│                                                          │
└─────────────────────────────────────────────────────────┘
```

### 2.2 Key Derivation (Argon2id)
```rust
// Parameters (OWASP recommended)
const MEMORY_COST: u32 = 65536;    // 64 MB
const TIME_COST: u32 = 3;          // 3 iterations
const PARALLELISM: u32 = 4;        // 4 threads
const OUTPUT_LENGTH: usize = 32;   // 256 bits

pub fn derive_key(password: &str, salt: &[u8]) -> [u8; 32] {
    let argon2 = Argon2::new(
        Algorithm::Argon2id,
        Version::Version13,
        Params::new(MEMORY_COST, TIME_COST, PARALLELISM, Some(OUTPUT_LENGTH))
            .expect("valid params")
    );
    
    let mut key = [0u8; 32];
    argon2.hash_password_into(password.as_bytes(), salt, &mut key)
        .expect("key derivation failed");
    
    key
}
```

### 2.3 Data Encryption (AES-256-GCM)
```rust
use aes_gcm::{Aes256Gcm, KeyInit, Nonce};
use aes_gcm::aead::Aead;

pub struct Encryptor {
    cipher: Aes256Gcm,
}

impl Encryptor {
    pub fn new(key: &[u8; 32]) -> Self {
        let cipher = Aes256Gcm::new_from_slice(key)
            .expect("invalid key length");
        Self { cipher }
    }
    
    pub fn encrypt(&self, plaintext: &[u8]) -> (Vec<u8>, [u8; 12]) {
        // Generate random nonce
        let mut nonce_bytes = [0u8; 12];
        rand::thread_rng().fill(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);
        
        // Encrypt
        let ciphertext = self.cipher
            .encrypt(nonce, plaintext)
            .expect("encryption failed");
        
        (ciphertext, nonce_bytes)
    }
    
    pub fn decrypt(&self, ciphertext: &[u8], nonce: &[u8; 12]) -> Vec<u8> {
        let nonce = Nonce::from_slice(nonce);
        
        self.cipher
            .decrypt(nonce, ciphertext)
            .expect("decryption failed")
    }
}
```

---

## 3. Memory Security

### 3.1 Sensitive Data Handling
```rust
use zeroize::Zeroize;

pub struct SecureBuffer {
    data: Vec<u8>,
}

impl SecureBuffer {
    pub fn new(data: Vec<u8>) -> Self {
        Self { data }
    }
    
    pub fn as_bytes(&self) -> &[u8] {
        &self.data
    }
}

impl Drop for SecureBuffer {
    fn drop(&mut self) {
        // Zeroize memory on drop
        self.data.zeroize();
    }
}
```

### 3.2 Memory Protection Rules
1. **Zeroize on Drop** - All sensitive data zeroized when dropped
2. **No Copy** - Sensitive types implement `!Copy`
3. **No Debug** - Sensitive types don't derive `Debug`
4. **No Serialize** - Sensitive types don't derive `Serialize`
5. **Stack Allocation** - Prefer stack over heap for small secrets

---

## 4. Authentication Security

### 4.1 Master Password
```
┌─────────────────────────────────────────────────────────┐
│                    Master Password Flow                   │
├─────────────────────────────────────────────────────────┤
│                                                          │
│  User Input                                              │
│      ↓                                                   │
│  ┌─────────────────────────────────────────────────┐   │
│  │  Validate Strength (length-first policy)         │   │
│  │  - Minimum 12 characters                         │   │
│  │  - Reject common passwords (top-10k list)        │   │
│  │  - Show strength meter (zxcvbn-style)            │   │
│  │  - No mandatory complexity rules                 │   │
│  └─────────────────────────────────────────────────┘   │
│      ↓                                                   │
│  ┌─────────────────────────────────────────────────┐   │
│  │  Derive Key                                      │   │
│  │  - Generate random salt (16 bytes)               │   │
│  │  - Argon2id(password, salt) → key                │   │
│  │  - Store salt + key hash in database             │   │
│  └─────────────────────────────────────────────────┘   │
│      ↓                                                   │
│  ┌─────────────────────────────────────────────────┐   │
│  │  Verify on Unlock                                │   │
│  │  - User enters password                          │   │
│  │  - Derive key with stored salt                   │   │
│  │  - Compare with stored hash                      │   │
│  │  - If match: unlock vault                        │   │
│  │  - If no match: increment attempt counter        │   │
│  │    + exponential backoff after 5 failed attempts │   │
│  └─────────────────────────────────────────────────┘   │
│                                                          │
└─────────────────────────────────────────────────────────┘
```

### 4.1.1 Master Password Policy

Following **NIST SP 800-63B** (2017+) recommendations: **length over complexity**.

| Rule | Required |
|------|----------|
| Minimum length | 12 characters |
| Maximum length | 128 characters |
| Mandatory uppercase | ❌ Not required |
| Mandatory lowercase | ❌ Not required |
| Mandatory digit | ❌ Not required |
| Mandatory special char | ❌ Not required |
| Reject common passwords | ✅ Top-10k list (e.g., `password123`) |
| Strength meter shown | ✅ zxcvbn or similar |
| Allow passphrases | ✅ Encouraged (e.g., "correct horse battery staple") |

**Rationale:**
- Forced complexity rules push users to predictable patterns (`Password1!`).
- Long passphrases have more entropy and are easier to remember.
- Reference: [NIST SP 800-63B §5.1.1.2](https://pages.nist.gov/800-63-3/sp800-63b.html#sec5).

### 4.1.2 No Recovery — Critical UX Rule

**ShellMate does NOT support master password recovery.** This is by design for a local-first vault: any recovery mechanism would be an attack vector.

**Required UX:**
- During vault setup, user must check a confirmation: "I understand that if I forget my master password, my data cannot be recovered."
- Suggest user write it down or store in a separate password manager
- Optional: offer to print/display a "vault info" sheet (vault location, hint they wrote — never the password itself)
- Strongly recommend exporting an **encrypted backup** (Phase 14) periodically

**On vault setup screen:**
```
⚠️  Important: There is no way to recover your master password.
   If you forget it, all stored credentials will be permanently lost.

   Tips:
   • Use a passphrase you'll remember (e.g., 4-5 random words)
   • Write it down and store it somewhere safe
   • Consider using a separate password manager

   [✓] I understand the risks. There is no recovery.

   [ Cancel ]    [ Create Vault ]
```

### 4.1.3 Brute-Force Mitigation
- Argon2id parameters tuned to ~500ms-1s on target hardware (configurable)
- Failed unlock attempts tracked in-memory (not persisted across app restart to avoid lockout from process kill)
- Exponential backoff: 1s, 2s, 5s, 10s, 30s after attempts 5, 6, 7, 8, 9+
- After 10 consecutive failed attempts in single session: optional 5-min cooldown (configurable in settings)

### 4.2 SSH Authentication
```
┌─────────────────────────────────────────────────────────┐
│                    SSH Auth Methods                       │
├─────────────────────────────────────────────────────────┤
│                                                          │
│  Method 1: Password                                      │
│  ┌─────────────────────────────────────────────────┐   │
│  │  - Retrieve encrypted password from vault        │   │
│  │  - Decrypt with vault key                        │   │
│  │  - Send to SSH server                            │   │
│  │  - Zeroize password after use                    │   │
│  └─────────────────────────────────────────────────┘   │
│                                                          │
│  Method 2: SSH Key                                      │
│  ┌─────────────────────────────────────────────────┐   │
│  │  - Retrieve encrypted private key from vault     │   │
│  │  - Decrypt with vault key                        │   │
│  │  - If key has passphrase:                        │   │
│  │    - Retrieve encrypted passphrase from vault    │   │
│  │    - Decrypt passphrase                          │   │
│  │    - Use passphrase to decrypt private key       │   │
│  │  - Send public key to SSH server                 │   │
│  │  - Zeroize private key after use                 │   │
│  └─────────────────────────────────────────────────┘   │
│                                                          │
└─────────────────────────────────────────────────────────┘
```

---

## 5. Database Security

### 5.1 SQL Injection Prevention
```rust
// ✅ Safe: Parameterized query
conn.execute(
    "SELECT * FROM hosts WHERE id = ?1",
    params![host_id],
)?;

// ❌ Unsafe: String interpolation (NEVER DO THIS)
let query = format!("SELECT * FROM hosts WHERE id = '{}'", host_id);
conn.execute(&query, [])?;
```

### 5.2 Data Validation
```rust
pub fn validate_host(host: &HostInput) -> Result<(), ValidationError> {
    // Validate hostname
    if host.hostname.is_empty() {
        return Err(ValidationError::EmptyHostname);
    }
    
    // Validate port
    if host.port < 1 || host.port > 65535 {
        return Err(ValidationError::InvalidPort);
    }
    
    // Validate username
    if host.username.is_empty() {
        return Err(ValidationError::EmptyUsername);
    }
    
    // Sanitize input
    if host.hostname.contains(';') || host.hostname.contains('|') {
        return Err(ValidationError::InvalidHostname);
    }
    
    Ok(())
}
```

---

## 6. Network Security

### 6.1 SSH Host Verification
```
┌─────────────────────────────────────────────────────────┐
│                    Host Verification                      │
├─────────────────────────────────────────────────────────┤
│                                                          │
│  First Connection                                        │
│  ┌─────────────────────────────────────────────────┐   │
│  │  1. Receive host key from server                 │   │
│  │  2. Display fingerprint to user                  │   │
│  │  3. Ask user to confirm                          │   │
│  │  4. Store in known_hosts                         │   │
│  └─────────────────────────────────────────────────┘   │
│                                                          │
│  Subsequent Connections                                  │
│  ┌─────────────────────────────────────────────────┐   │
│  │  1. Receive host key from server                 │   │
│  │  2. Compare with stored key                      │   │
│  │  3. If match: proceed                            │   │
│  │  4. If mismatch: warn user (MITM attack?)        │   │
│  └─────────────────────────────────────────────────┘   │
│                                                          │
└─────────────────────────────────────────────────────────┘
```

### 6.2 Connection Security
- **Keepalive:** Prevent timeout disconnections
- **Timeout:** Configurable connection timeout
- **Retry:** Automatic reconnection with backoff
- **Rate Limiting:** Prevent brute force attempts

---

## 7. Vault Security

### 7.1 Vault States
```
┌─────────────────────────────────────────────────────────┐
│                    Vault State Machine                    │
├─────────────────────────────────────────────────────────┤
│                                                          │
│  ┌──────────┐     ┌──────────┐     ┌──────────┐        │
│  │  Setup   │────>│ Unlocked │────>│  Locked  │        │
│  └──────────┘     └──────────┘     └──────────┘        │
│       │                │                  │             │
│       │                │                  │             │
│       ▼                ▼                  ▼             │
│  ┌──────────┐     ┌──────────┐     ┌──────────┐        │
│  │ Created  │     │  Active  │     │  Idle    │        │
│  └──────────┘     └──────────┘     └──────────┘        │
│                                                          │
│  Transitions:                                            │
│  - Setup → Unlocked: First time setup                    │
│  - Unlocked → Locked: Manual lock or timeout             │
│  - Locked → Unlocked: Correct master password            │
│                                                          │
└─────────────────────────────────────────────────────────┘
```

### 7.2 Auto-Lock
```rust
pub struct Vault {
    is_unlocked: bool,
    last_activity: Instant,
    lock_timeout: Duration,
}

impl Vault {
    pub fn check_auto_lock(&mut self) {
        if self.is_unlocked && self.last_activity.elapsed() > self.lock_timeout {
            self.lock();
        }
    }
    
    pub fn record_activity(&mut self) {
        self.last_activity = Instant::now();
    }
    
    pub fn lock(&mut self) {
        self.is_unlocked = false;
        // Clear encryption key from memory
        self.clear_key();
    }
}
```

---

## 8. Logging Security

### 8.1 What to Log
✅ **Safe to Log:**
- Connection attempts (success/failure)
- Host operations (add, edit, delete)
- Vault lock/unlock events
- Errors (without sensitive data)

❌ **Never Log:**
- Passwords
- Private keys
- Master password
- Encryption keys
- Session tokens

### 8.2 Log Sanitization
```rust
pub fn sanitize_log(message: &str) -> String {
    // Remove potential sensitive patterns
    let patterns = [
        r"(?i)password\s*[:=]\s*\S+",
        r"(?i)key\s*[:=]\s*\S+",
        r"(?i)secret\s*[:=]\s*\S+",
        r"-----BEGIN.*PRIVATE KEY-----",
    ];
    
    let mut sanitized = message.to_string();
    for pattern in patterns {
        sanitized = sanitized.replace(pattern, "[REDACTED]");
    }
    
    sanitized
}
```

---

## 9. Security Checklist

### 9.1 Development Checklist
- [ ] All credentials encrypted at rest
- [ ] No plaintext passwords in code
- [ ] Parameterized SQL queries only
- [ ] Input validation on all user inputs
- [ ] Sensitive data zeroized after use
- [ ] No sensitive data in logs
- [ ] SSH host verification enabled
- [ ] Vault auto-lock implemented
- [ ] Error messages don't leak secrets
- [ ] No telemetry/analytics

### 9.2 Testing Checklist
- [ ] Penetration testing
- [ ] SQL injection testing
- [ ] Buffer overflow testing
- [ ] Memory leak testing
- [ ] Credential exposure testing
- [ ] Network security testing

### 9.3 Release Checklist
- [ ] Security audit completed
- [ ] Dependencies updated
- [ ] No known vulnerabilities
- [ ] Security documentation updated

---

## 10. Incident Response

### 10.1 Security Incident Process
1. **Identify** - Detect and confirm incident
2. **Contain** - Isolate affected systems
3. **Eradicate** - Remove threat
4. **Recover** - Restore normal operations
5. **Learn** - Document and improve

### 10.2 Vulnerability Reporting
- Security issues reported via GitHub Security Advisories
- Critical vulnerabilities patched within 24 hours
- Users notified via release notes

---

## 11. Encryption Strategy

### 11.1 Decision: Defense in Depth (Both Layers)

ShellMate v1.0 uses **two layers of at-rest encryption**:

1. **Per-credential AES-256-GCM** — passwords, private keys, passphrases stored in `credentials` table are encrypted with the vault key (Argon2id-derived). This was implemented in Phase 2.

2. **SQLCipher full-DB encryption** — the entire SQLite database file is encrypted with a separate DB key (also derived from the master password via HKDF, distinct from the vault key). All metadata — hostnames, usernames, group names, snippet contents, settings — protected at rest. Implemented in Phase 7.

Both layers are active simultaneously in v1.0.

### 11.2 Why Both?

| Concern | Per-credential AES-GCM | SQLCipher full-DB |
|---------|------------------------|-------------------|
| Credential theft from stolen DB file | ✅ blocks | ✅ blocks |
| Metadata leakage from stolen DB file | ❌ visible | ✅ blocks |
| Memory dump while vault locked | ✅ blocks | ✅ blocks |
| Memory dump while vault unlocked | ❌ key in mem | ❌ key in mem |
| Granular re-encryption (rotate vault) | ✅ per-cred re-encrypt | ⚠️ requires DB rebuild |
| Defense if one layer is compromised | — | acts as backstop |

**Rationale:** Per-credential layer survives even if SQLCipher key is leaked or weakly chosen. SQLCipher protects metadata that per-credential layer doesn't reach. Belt-and-suspenders.

### 11.3 Implementation (Phase 7)

- Migration tool: detects existing plaintext SQLite (Phase 1-6 builds), prompts user, atomically rebuilds DB with SQLCipher
- DB key derivation: separate output from same master password via HKDF — not the same byte-string as vault key
- `PRAGMA key` set on connection open
- Per-credential encryption stays in place — no changes to `credentials` table format

### 11.4 Performance

Target: < 15% read/write regression vs plaintext SQLite (CI gate). SQLCipher uses AES-256 in CBC mode with HMAC-SHA512 by default. Acceptable for typical query workload (host list, snippet search, audit query).

### 11.5 Key Hierarchy

```
Master Password
       │
       ▼ Argon2id (per Phase 2 §2.2)
Master Key (256-bit derived)
       │
       ├──────────────────────────┐
       ▼                          ▼
   HKDF-SHA256              HKDF-SHA256
   info: "vault.v1"         info: "db.v1"
       │                          │
       ▼                          ▼
   Vault Key                 DB Key
   (encrypts                 (SQLCipher
    credentials)              PRAGMA key)
```

Domain separation via HKDF `info` parameter prevents either key from being reused for the other context.

---

## 12. Biometric Unlock Security (Phase 8)

### 12.1 Threat Model

Biometric unlock provides **convenience**, not extra security beyond the master password. The master password is still the root of trust.

### 12.2 Architecture

```
First-time enable
─────────────────
   Master Password
        │
        ▼ Argon2id
   Master Key
        │
        ▼ wrap with biometric-protected key
   Wrapped Master Key  ──> stored in OS secure enclave
                          (TPM / Secure Enclave / Keystore)

Subsequent unlock
─────────────────
   User biometric prompt
        │
        ▼ OS verifies, releases biometric-protected key
   Biometric Key
        │
        ▼ unwrap
   Master Key  ──> derive vault & DB keys
```

### 12.3 Per-OS Implementation

| Platform | Backing Store | API |
|----------|---------------|-----|
| macOS | Keychain + Secure Enclave | `LocalAuthentication` framework |
| iOS | Keychain + Secure Enclave | `LocalAuthentication` framework |
| Windows | Windows Hello + TPM | `Windows.Security.Credentials.UI` |
| Android | Android Keystore | `BiometricPrompt` |
| Linux | Not supported (fallback to master password) | — |

### 12.4 Security Rules

- Biometric unlock is **per-device only** (not synced)
- Failed biometric attempts do NOT increment master password lockout counter
- After 5 consecutive biometric failures, fall back to master password
- Disabling biometric removes the wrapped master key
- Master password change invalidates wrapped master key — biometric must be re-enrolled

---

## 13. Sync Security (Phase 9)

### 13.1 Threat Model

Cloud provider (iCloud, GDrive, Dropbox, S3, WebDAV server) is treated as **untrusted**. Sync payload must be unreadable to the provider.

### 13.2 Encryption

```
Local entity (host config / snippet / setting)
       │
       ▼ serialize to canonical JSON
       │
       ▼ encrypt with sync key (HKDF info: "sync.v1")
       │
       ▼ AES-256-GCM with random nonce
   Encrypted Payload
       │
       ▼ upload via backend adapter
   Cloud storage object (opaque ID, no metadata in path)
```

### 13.3 No Metadata Leakage

- Object names use random UUIDs, not host labels
- Manifest itself is encrypted
- HTTP headers (where controllable) sanitized
- Backend adapter must not log payload contents

### 13.4 Verification

Manual test required for v1.0 GA: configure each backend, push data, verify with provider's CLI/web UI that no plaintext is visible. Document the test in security review notes.

### 13.5 Conflict Resolution Without Decryption Leak

Conflicts resolved client-side after decryption. Cloud provider only sees opaque payloads with version vectors (which are themselves encrypted in the manifest).

---

## 14. Team Vault Security (Phase 11)

### 14.1 Key Hierarchy

```
Team Master Key (random, 256-bit, generated on team creation)
       │
       ├─ wrapped with each member's public key (X25519)
       │  └─ stored in shared sync object
       │
       └─ used to encrypt shared host configs (AES-256-GCM)
```

### 14.2 Member Lifecycle

- **Add member**: encrypt team master key with their public key, append to wrapped-keys list
- **Revoke member**: rotate team master key, re-encrypt all shared hosts, distribute new wrapped keys
- **Revocation is reactive** — already-extracted data cannot be unsent. UI must warn explicitly.

### 14.3 Trust Model

- Each member has personal vault (their own master password)
- Personal vault holds X25519 keypair for team participation
- Team master key never leaves encrypted form except briefly in member RAM
- No "team admin" with God-mode key — quorum or single-creator model

### 14.4 Conflict on Shared Hosts

Two team members editing same host: last-write-wins by default with timestamp; manual merge UI for explicit conflicts. Audit log captures who-changed-what.

---

## 15. Plugin Security (Phase 12)

### 15.1 Sandbox

Plugins run in **Wasmtime** WASM sandbox:
- No native code execution
- No direct OS syscalls
- No filesystem or network access by default

### 15.2 Capability-Based Permissions

Plugins declare capabilities in signed manifest. User reviews on install. Capabilities revocable.

| Capability | Risk | UX |
|-----------|------|-----|
| `log` | None | Auto-granted |
| `terminal_data` | Could exfiltrate session content | Per-install consent, can revoke |
| `panel` | UI surface only, no data access | Per-install consent |
| `network` | Allow-listed hosts only, outbound HTTP | Per-install with explicit host list |
| `filesystem` | Scoped to `~/Documents/Plugins/<id>/` | Per-install consent |
| `secrets` | Read vault credentials | Per-install + per-access prompt UI |

### 15.3 Plugin Manifest Signing

- Manifests must be Ed25519-signed by plugin author
- Public key shipped in manifest, hash displayed to user on install
- Update path: signature must match same key, otherwise treated as new install

### 15.4 Crash Isolation

Plugin panic / trap does NOT crash host app. Errors logged, plugin disabled until user re-enables.

---

## 16. Audit Log Security (Phase 13)

### 16.1 Storage

Audit log stored in dedicated SQLite table (`audit_events`). Each row encrypted with vault key (same as credentials).

### 16.2 Hash Chain

Each event includes hash of previous event:
```
event_n.prev_hash = SHA256(event_{n-1} canonical bytes)
```
Tampering with any past event invalidates all subsequent hashes — detected on export verification.

### 16.3 Privacy

- Opt-in **per host** (default OFF)
- Command history opt-in separately (default OFF — high sensitivity)
- Redaction patterns applied before storage (regex, configurable, e.g., `password=\S+`)
- Retention configurable (default 90 days, max "forever")

### 16.4 Export

`Export Audit Log` produces signed JSONL:
- Each line is a canonicalized event
- Last line is signature of file hash
- Compliance evidence-grade output

---

*This document outlines the complete security architecture and practices for ShellMate.*
