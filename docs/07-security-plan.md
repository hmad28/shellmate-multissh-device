# Security Plan
## ShellMate - Security Architecture

**Version:** 1.0
**Last Updated:** 2026-06-07

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
│  │  Validate Strength                               │   │
│  │  - Minimum 8 characters                          │   │
│  │  - At least 1 uppercase                          │   │
│  │  - At least 1 lowercase                          │   │
│  │  - At least 1 number                             │   │
│  │  - At least 1 special character                  │   │
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
│  └─────────────────────────────────────────────────┘   │
│                                                          │
└─────────────────────────────────────────────────────────┘
```

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

*This document outlines the complete security architecture and practices for ShellMate.*
