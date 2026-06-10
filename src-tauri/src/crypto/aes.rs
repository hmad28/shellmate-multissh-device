use crate::errors::{AppError, AppResult};
use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use rand::RngCore;
use serde::{Deserialize, Serialize};

pub const NONCE_LEN: usize = 12;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedBlob {
    pub ciphertext: Vec<u8>,
    pub nonce: [u8; NONCE_LEN],
}

/// Encrypt `plaintext` with AES-256-GCM using a randomly generated nonce.
pub fn encrypt(key: &[u8; 32], plaintext: &[u8]) -> AppResult<EncryptedBlob> {
    let cipher = Aes256Gcm::new_from_slice(key)
        .map_err(|e| AppError::Internal(format!("invalid AES key: {e}")))?;

    let mut nonce_bytes = [0u8; NONCE_LEN];
    rand::thread_rng().fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, plaintext)
        .map_err(|e| AppError::Internal(format!("AES-GCM encrypt failed: {e}")))?;

    Ok(EncryptedBlob {
        ciphertext,
        nonce: nonce_bytes,
    })
}

/// Decrypt an `EncryptedBlob` using the provided key. Authenticated — fails on tamper.
pub fn decrypt(key: &[u8; 32], blob: &EncryptedBlob) -> AppResult<Vec<u8>> {
    let cipher = Aes256Gcm::new_from_slice(key)
        .map_err(|e| AppError::Internal(format!("invalid AES key: {e}")))?;
    let nonce = Nonce::from_slice(&blob.nonce);
    cipher
        .decrypt(nonce, blob.ciphertext.as_ref())
        .map_err(|_| AppError::InvalidInput("decryption failed (wrong key or tampered data)".into()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_succeeds() {
        let key = [42u8; 32];
        let blob = encrypt(&key, b"super secret").unwrap();
        let plain = decrypt(&key, &blob).unwrap();
        assert_eq!(plain, b"super secret");
    }

    #[test]
    fn wrong_key_fails() {
        let blob = encrypt(&[1u8; 32], b"hi").unwrap();
        assert!(decrypt(&[2u8; 32], &blob).is_err());
    }

    #[test]
    fn tampered_ciphertext_fails() {
        let key = [7u8; 32];
        let mut blob = encrypt(&key, b"hi").unwrap();
        blob.ciphertext[0] ^= 0xff;
        assert!(decrypt(&key, &blob).is_err());
    }
}
