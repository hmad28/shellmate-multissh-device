# Security Plan
## ShellMate - Security Architecture

**Version:** 1.1
**Last Updated:** 2026-06-09

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
- Strongly recommend exporting an **encrypted backup** (post-MVP feature) periodically

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

## 11. Encryption Strategy Decision

### 11.1 Decision: Per-Field Encryption (MVP)

For MVP, ShellMate uses **per-field encryption** of credentials only — passwords, private keys, and passphrases stored in the `credentials` table are encrypted with AES-256-GCM. Other tables (`hosts`, `groups`, `snippets`, `port_forwards`, `settings`) are stored as plaintext SQLite rows.

### 11.2 What This Protects
- ✅ Passwords and private keys at rest
- ✅ Memory dump while vault is locked
- ✅ Stolen SQLite file cannot be used to authenticate to SSH servers
- ✅ Granular re-encryption (rotate vault key without re-encrypting whole DB)

### 11.3 What This Does NOT Protect
- ❌ Hostnames, usernames, host labels, group names, snippet contents, notes
- ❌ Metadata leakage if attacker accesses SQLite file (they can see your server inventory)
- ❌ Snippet contents may include sensitive paths or commands

For MVP threat model (single-user device, file-system-level protection from OS), this trade-off is acceptable. Users with stricter requirements should rely on full-disk encryption (BitLocker, FileVault, LUKS).

### 11.4 Alternative Considered: SQLCipher

**SQLCipher** would encrypt the entire SQLite database file, protecting all metadata. It is a transparent layer over SQLite with strong encryption (AES-256, PBKDF2 by default).

| Aspect | Per-Field (MVP) | SQLCipher (Future) |
|--------|-----------------|--------------------|
| Metadata protection | ❌ | ✅ |
| Performance overhead | None | ~5-15% on read/write |
| Build complexity | Low (pure Rust) | Medium (C dep, needs `rusqlite` SQLCipher feature) |
| Cross-platform | Easy | Requires platform-specific build flags |
| Key management | Per-vault (Argon2id-derived) | Same key, applied at DB open |
| Migration cost from MVP | — | Need migration script |

### 11.5 Post-MVP Migration Path

Re-evaluate SQLCipher when:
- User requests metadata privacy (stronger threat model)
- Multi-device sync is added (encrypted backups in untrusted cloud)
- Performance budget allows ~10% read/write overhead

**Migration approach:**
1. Add SQLCipher as opt-in setting ("Full database encryption")
2. On enable: derive new DB key from master password (separate from credential vault key, or shared via HKDF)
3. Re-create database with `PRAGMA key`, copy data, swap files atomically
4. Existing per-field encryption stays — defense in depth

### 11.6 Implementation Notes (MVP)

```rust
// credentials table: encrypted blob + nonce
struct EncryptedCredential {
    id: String,
    cred_type: CredentialType,
    encrypted_data: Vec<u8>,  // AES-256-GCM ciphertext + auth tag
    nonce: [u8; 12],          // GCM nonce (random per encryption)
    created_at: String,
    updated_at: String,
}

// Decrypt only when needed (e.g., during SSH connect), zeroize after
fn use_credential(id: &str, vault_key: &[u8; 32]) -> Result<()> {
    let row = db::get_credential(id)?;
    let plaintext = decrypt(&row.encrypted_data, &row.nonce, vault_key)?;
    let secret = SecureBuffer::new(plaintext);
    // ... use secret ...
    // SecureBuffer::Drop zeroizes
    Ok(())
}
```

**Key rules:**
- Vault key lives only in memory (never persisted)
- Generate fresh nonce for every encryption (12 random bytes)
- Use authenticated encryption (GCM) — never raw AES-CBC or AES-CTR
- Rotate vault key when master password changes (re-encrypt all credentials)

---

*This document outlines the complete security architecture and practices for ShellMate.*
