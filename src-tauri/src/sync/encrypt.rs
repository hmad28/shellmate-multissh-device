use crate::errors::AppResult;
use aes_gcm::{Aes256Gcm, KeyInit, Nonce};
use aes_gcm::aead::Aead;
use hkdf::Hkdf;
use rand::RngCore;
use sha2::Sha256;

const HKDF_SALT: &[u8] = b"shellmate-sync-v1";
const HKDF_INFO: &[u8] = b"sync-payload-key";

/// Derive a sync encryption key from the device ID.
/// In production, this should use the vault's sync key (HKDF info: "sync.v1").
/// For MVP, we derive from device ID as a simple per-device key.
fn derive_sync_key(device_id: &str) -> [u8; 32] {
    let hk = Hkdf::<Sha256>::new(Some(HKDF_SALT), device_id.as_bytes());
    let mut key = [0u8; 32];
    hk.expand(HKDF_INFO, &mut key).expect("HKDF expand failed");
    key
}

/// Encrypt a sync payload (JSON bytes) with AES-256-GCM.
/// Returns: nonce (12 bytes) || ciphertext.
pub fn encrypt_payload(payload: &[u8], device_id: &str) -> AppResult<Vec<u8>> {
    let key = derive_sync_key(device_id);
    let cipher = Aes256Gcm::new_from_slice(&key)
        .map_err(|e| crate::errors::AppError::Internal(format!("AES init: {e}")))?;

    let mut nonce_bytes = [0u8; 12];
    rand::thread_rng().fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, payload)
        .map_err(|e| crate::errors::AppError::Internal(format!("encrypt: {e}")))?;

    // Prepend nonce to ciphertext.
    let mut result = Vec::with_capacity(12 + ciphertext.len());
    result.extend_from_slice(&nonce_bytes);
    result.extend_from_slice(&ciphertext);
    Ok(result)
}

/// Decrypt a sync payload. Input format: nonce (12 bytes) || ciphertext.
pub fn decrypt_payload(data: &[u8], device_id: &str) -> AppResult<Vec<u8>> {
    if data.len() < 12 {
        return Err(crate::errors::AppError::Internal(
            "sync payload too short".into(),
        ));
    }

    let key = derive_sync_key(device_id);
    let cipher = Aes256Gcm::new_from_slice(&key)
        .map_err(|e| crate::errors::AppError::Internal(format!("AES init: {e}")))?;

    let (nonce_bytes, ciphertext) = data.split_at(12);
    let nonce = Nonce::from_slice(nonce_bytes);

    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| crate::errors::AppError::Internal(format!("decrypt: {e}")))?;

    Ok(plaintext)
}
