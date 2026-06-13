use crate::errors::{AppError, AppResult};
use argon2::{Algorithm, Argon2, Params, Version};
use hkdf::Hkdf;
use rand::RngCore;
use sha2::Sha256;

/// Argon2id parameters tuned per OWASP guidance, balanced for desktop UX (~500ms-1s).
/// Memory cost is the dominant security factor; do not lower without review.
pub const KEY_LEN: usize = 32; // 256-bit
pub const SALT_LEN: usize = 16; // 128-bit
pub const ARGON2_MEMORY_KIB: u32 = 65_536; // 64 MiB
pub const ARGON2_TIME_COST: u32 = 3;
pub const ARGON2_PARALLELISM: u32 = 4;

pub const ARGON2_PARAMS: ArgonParams = ArgonParams {
    memory_kib: ARGON2_MEMORY_KIB,
    time_cost: ARGON2_TIME_COST,
    parallelism: ARGON2_PARALLELISM,
    output_len: KEY_LEN,
};

#[derive(Debug, Clone, Copy)]
pub struct ArgonParams {
    pub memory_kib: u32,
    pub time_cost: u32,
    pub parallelism: u32,
    pub output_len: usize,
}

/// Derive a 32-byte key from `password` and `salt` using Argon2id.
pub fn derive_key(password: &[u8], salt: &[u8]) -> AppResult<[u8; KEY_LEN]> {
    if salt.len() != SALT_LEN {
        return Err(AppError::Internal(format!(
            "salt must be {SALT_LEN} bytes, got {}",
            salt.len()
        )));
    }

    let params = Params::new(
        ARGON2_PARAMS.memory_kib,
        ARGON2_PARAMS.time_cost,
        ARGON2_PARAMS.parallelism,
        Some(ARGON2_PARAMS.output_len),
    )
    .map_err(|e| AppError::Internal(format!("invalid argon2 params: {e}")))?;

    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);

    let mut out = [0u8; KEY_LEN];
    argon2
        .hash_password_into(password, salt, &mut out)
        .map_err(|e| AppError::Internal(format!("argon2 derivation failed: {e}")))?;
    Ok(out)
}

/// Generate a fresh random salt suitable for `derive_key`.
pub fn generate_salt() -> [u8; SALT_LEN] {
    let mut salt = [0u8; SALT_LEN];
    rand::thread_rng().fill_bytes(&mut salt);
    salt
}

/// Derive two independent 32-byte keys from a single master key using HKDF.
/// - `vault_key`: used for per-credential AES-256-GCM encryption
/// - `db_key`: used as the SQLCipher database encryption key
pub fn derive_vault_and_db_keys(master_key: &[u8; KEY_LEN]) -> ([u8; KEY_LEN], [u8; KEY_LEN]) {
    let hk = Hkdf::<Sha256>::new(Some(b"shellmate-v1"), master_key);
    let mut vault_key = [0u8; KEY_LEN];
    let mut db_key = [0u8; KEY_LEN];
    hk.expand(b"vault-key", &mut vault_key).expect("HKDF expand failed");
    hk.expand(b"db-key", &mut db_key).expect("HKDF expand failed");
    (vault_key, db_key)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derive_key_is_deterministic() {
        let salt = [0u8; SALT_LEN];
        let a = derive_key(b"password", &salt).unwrap();
        let b = derive_key(b"password", &salt).unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn derive_key_changes_with_salt() {
        let a = derive_key(b"password", &[0u8; SALT_LEN]).unwrap();
        let b = derive_key(b"password", &[1u8; SALT_LEN]).unwrap();
        assert_ne!(a, b);
    }
}
