use crate::errors::{AppError, AppResult};
use aes_gcm::aead::Aead;
use aes_gcm::{Aes256Gcm, KeyInit, Nonce};
use hkdf::Hkdf;
use rand::RngCore;
use sha2::Sha256;

const DEVICE_SECRET_LEN: usize = 32;
const HKDF_SALT: &[u8] = b"shellmate-biometric-v1";
const HKDF_INFO: &[u8] = b"biometric-wrap-key";

/// Biometric provider interface. Each platform implements this trait.
pub trait BiometricProvider: Send + Sync {
    /// Check if biometric authentication is available on this device.
    fn is_available(&self) -> bool;

    /// Prompt the user for biometric verification. Returns true if verified.
    fn verify_user(&self, reason: &str) -> bool;
}

/// Standard biometric provider for platforms without native integration.
/// Always returns false — biometric not supported.
pub struct StubProvider;

impl BiometricProvider for StubProvider {
    fn is_available(&self) -> bool {
        false
    }

    fn verify_user(&self, _reason: &str) -> bool {
        false
    }
}

/// Create the platform-appropriate biometric provider.
pub fn create_provider() -> Box<dyn BiometricProvider> {
    #[cfg(target_os = "windows")]
    {
        Box::new(windows::WindowsHelloProvider)
    }
    #[cfg(not(target_os = "windows"))]
    {
        Box::new(StubProvider)
    }
}

/// Derive a wrapping key from the device secret using HKDF.
fn derive_wrap_key(device_secret: &[u8; DEVICE_SECRET_LEN]) -> [u8; 32] {
    let hk = Hkdf::<Sha256>::new(Some(HKDF_SALT), device_secret);
    let mut key = [0u8; 32];
    hk.expand(HKDF_INFO, &mut key).expect("HKDF expand failed");
    key
}

/// Wrap (encrypt) the master key using a device secret.
/// Returns (wrapped_master_key, nonce).
pub fn wrap_master_key(
    master_key: &[u8; 32],
    device_secret: &[u8; DEVICE_SECRET_LEN],
) -> AppResult<(Vec<u8>, [u8; 12])> {
    let wrap_key = derive_wrap_key(device_secret);
    let cipher = Aes256Gcm::new_from_slice(&wrap_key)
        .map_err(|e| AppError::Internal(format!("AES init failed: {e}")))?;

    let mut nonce_bytes = [0u8; 12];
    rand::thread_rng().fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, master_key.as_ref())
        .map_err(|e| AppError::Internal(format!("encrypt failed: {e}")))?;

    Ok((ciphertext, nonce_bytes))
}

/// Unwrap (decrypt) the master key using a device secret.
pub fn unwrap_master_key(
    wrapped: &[u8],
    nonce: &[u8; 12],
    device_secret: &[u8; DEVICE_SECRET_LEN],
) -> AppResult<[u8; 32]> {
    let wrap_key = derive_wrap_key(device_secret);
    let cipher = Aes256Gcm::new_from_slice(&wrap_key)
        .map_err(|e| AppError::Internal(format!("AES init failed: {e}")))?;

    let nonce = Nonce::from_slice(nonce);
    let plaintext = cipher
        .decrypt(nonce, wrapped)
        .map_err(|e| AppError::Internal(format!("decrypt failed: {e}")))?;

    if plaintext.len() != 32 {
        return Err(AppError::Internal("unexpected master key length".into()));
    }
    let mut key = [0u8; 32];
    key.copy_from_slice(&plaintext);
    Ok(key)
}

/// Generate a fresh random device secret.
pub fn generate_device_secret() -> [u8; DEVICE_SECRET_LEN] {
    let mut secret = [0u8; DEVICE_SECRET_LEN];
    rand::thread_rng().fill_bytes(&mut secret);
    secret
}

#[cfg(target_os = "windows")]
pub mod windows;
