use crate::errors::AppResult;
use aes_gcm::{Aes256Gcm, KeyInit, Nonce};
use aes_gcm::aead::Aead;
use hkdf::Hkdf;
use rand::RngCore;
use sha2::Sha256;

const HKDF_SALT: &[u8] = b"shellmate-sync-v1";
const HKDF_INFO_CRED: &[u8] = b"sync-credential-key";
const HKDF_INFO_PAYLOAD: &[u8] = b"sync-payload-key";

/// Derive a sync encryption key from the vault's master key material.
/// Uses HKDF with domain separation for credential vs payload encryption.
fn derive_key_from_master(master_key: &[u8], purpose: &[u8]) -> [u8; 32] {
    let hk = Hkdf::<Sha256>::new(Some(HKDF_SALT), master_key);
    let mut key = [0u8; 32];
    hk.expand(purpose, &mut key).expect("HKDF expand failed");
    key
}

/// Encrypt sync credentials (S3 keys, bearer tokens, etc.) with a key
/// derived from the vault master key.
pub fn encrypt_credentials(plaintext: &[u8], master_key: &[u8]) -> AppResult<(Vec<u8>, [u8; 12])> {
    let key = derive_key_from_master(master_key, HKDF_INFO_CRED);
    let cipher = Aes256Gcm::new_from_slice(&key)
        .map_err(|e| crate::errors::AppError::Internal(format!("AES init: {e}")))?;

    let mut nonce_bytes = [0u8; 12];
    rand::thread_rng().fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, plaintext)
        .map_err(|e| crate::errors::AppError::Internal(format!("encrypt: {e}")))?;

    Ok((ciphertext, nonce_bytes))
}

/// Decrypt sync credentials.
pub fn decrypt_credentials(ciphertext: &[u8], nonce: &[u8; 12], master_key: &[u8]) -> AppResult<Vec<u8>> {
    let key = derive_key_from_master(master_key, HKDF_INFO_CRED);
    let cipher = Aes256Gcm::new_from_slice(&key)
        .map_err(|e| crate::errors::AppError::Internal(format!("AES init: {e}")))?;

    let nonce = Nonce::from_slice(nonce);
    cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| crate::errors::AppError::Internal(format!("decrypt: {e}")))
}

/// Derive a sync payload encryption key from the vault master key.
/// This key is the SAME across all devices that share the same vault,
/// enabling cross-device decryption.
pub fn derive_sync_payload_key(master_key: &[u8]) -> [u8; 32] {
    derive_key_from_master(master_key, HKDF_INFO_PAYLOAD)
}

/// Encrypt a sync payload (JSON bytes) with AES-256-GCM.
/// Returns: nonce (12 bytes) || ciphertext.
pub fn encrypt_payload(payload: &[u8], sync_key: &[u8; 32]) -> AppResult<Vec<u8>> {
    let cipher = Aes256Gcm::new_from_slice(sync_key)
        .map_err(|e| crate::errors::AppError::Internal(format!("AES init: {e}")))?;

    let mut nonce_bytes = [0u8; 12];
    rand::thread_rng().fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, payload)
        .map_err(|e| crate::errors::AppError::Internal(format!("encrypt: {e}")))?;

    let mut result = Vec::with_capacity(12 + ciphertext.len());
    result.extend_from_slice(&nonce_bytes);
    result.extend_from_slice(&ciphertext);
    Ok(result)
}

/// Decrypt a sync payload. Input format: nonce (12 bytes) || ciphertext.
pub fn decrypt_payload(data: &[u8], sync_key: &[u8; 32]) -> AppResult<Vec<u8>> {
    if data.len() < 12 {
        return Err(crate::errors::AppError::Internal(
            "sync payload too short".into(),
        ));
    }

    let cipher = Aes256Gcm::new_from_slice(sync_key)
        .map_err(|e| crate::errors::AppError::Internal(format!("AES init: {e}")))?;

    let (nonce_bytes, ciphertext) = data.split_at(12);
    let nonce = Nonce::from_slice(nonce_bytes);

    cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| crate::errors::AppError::Internal(format!("decrypt: {e}")))
}
