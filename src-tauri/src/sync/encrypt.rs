use crate::errors::AppResult;
use aes_gcm::aead::Aead;
use aes_gcm::{Aes256Gcm, KeyInit, Nonce};
use hkdf::Hkdf;
use rand::RngCore;
use sha2::Sha256;

const HKDF_SALT: &[u8] = b"shellmate-sync-v1";
const HKDF_INFO_CRED: &[u8] = b"sync-credential-key";
const HKDF_INFO_PAYLOAD: &[u8] = b"sync-payload-key";

fn derive_key_from_master(master_key: &[u8], purpose: &[u8]) -> [u8; 32] {
    let hk = Hkdf::<Sha256>::new(Some(HKDF_SALT), master_key);
    let mut key = [0u8; 32];
    hk.expand(purpose, &mut key).expect("HKDF expand failed");
    key
}

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

pub fn decrypt_credentials(
    ciphertext: &[u8],
    nonce: &[u8; 12],
    master_key: &[u8],
) -> AppResult<Vec<u8>> {
    let key = derive_key_from_master(master_key, HKDF_INFO_CRED);
    let cipher = Aes256Gcm::new_from_slice(&key)
        .map_err(|e| crate::errors::AppError::Internal(format!("AES init: {e}")))?;

    let nonce = Nonce::from_slice(nonce);
    cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| crate::errors::AppError::Internal(format!("decrypt: {e}")))
}

pub fn derive_sync_payload_key(master_key: &[u8]) -> [u8; 32] {
    derive_key_from_master(master_key, HKDF_INFO_PAYLOAD)
}

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let key = [42u8; 32];
        let plaintext = b"hello, world!";
        let encrypted = encrypt_payload(plaintext, &key).unwrap();
        let decrypted = decrypt_payload(&encrypted, &key).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_wrong_key_fails() {
        let key1 = [1u8; 32];
        let key2 = [2u8; 32];
        let plaintext = b"secret data";
        let encrypted = encrypt_payload(plaintext, &key1).unwrap();
        assert!(decrypt_payload(&encrypted, &key2).is_err());
    }

    #[test]
    fn test_short_data_fails() {
        let key = [0u8; 32];
        assert!(decrypt_payload(&[0u8; 5], &key).is_err());
    }

    #[test]
    fn test_different_nonces() {
        let key = [0u8; 32];
        let plaintext = b"same plaintext";
        let enc1 = encrypt_payload(plaintext, &key).unwrap();
        let enc2 = encrypt_payload(plaintext, &key).unwrap();
        assert_ne!(enc1, enc2);
        assert_eq!(decrypt_payload(&enc1, &key).unwrap(), plaintext);
        assert_eq!(decrypt_payload(&enc2, &key).unwrap(), plaintext);
    }

    #[test]
    fn test_derive_sync_payload_key_deterministic() {
        let master = [99u8; 32];
        let key1 = derive_sync_payload_key(&master);
        let key2 = derive_sync_payload_key(&master);
        assert_eq!(key1, key2);
    }

    #[test]
    fn test_encrypt_decrypt_credentials() {
        let master = [77u8; 32];
        let creds = b"{'access_key': 'AKIA...', 'secret_key': '...'}";
        let (ct, nonce) = encrypt_credentials(creds, &master).unwrap();
        let decrypted = decrypt_credentials(&ct, &nonce, &master).unwrap();
        assert_eq!(decrypted, creds);
    }

    #[test]
    fn test_wrong_master_key_fails_credentials() {
        let master1 = [1u8; 32];
        let master2 = [2u8; 32];
        let creds = b"secret credentials";
        let (ct, nonce) = encrypt_credentials(creds, &master1).unwrap();
        assert!(decrypt_credentials(&ct, &nonce, &master2).is_err());
    }

    #[test]
    fn test_different_master_keys_different_payload_keys() {
        let m1 = [1u8; 32];
        let m2 = [2u8; 32];
        let k1 = derive_sync_payload_key(&m1);
        let k2 = derive_sync_payload_key(&m2);
        assert_ne!(k1, k2);
    }

    #[test]
    fn test_payload_encrypt_decrypt_with_sync_key() {
        let master = [55u8; 32];
        let key = derive_sync_payload_key(&master);
        let data = b"host config data";
        let encrypted = encrypt_payload(data, &key).unwrap();
        let decrypted = decrypt_payload(&encrypted, &key).unwrap();
        assert_eq!(decrypted, data);
    }
}
