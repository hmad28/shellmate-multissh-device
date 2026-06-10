use zeroize::Zeroize;

/// Wrapper around `Vec<u8>` that zeroizes on drop.
///
/// Used for sensitive data such as decrypted credentials and derived keys.
/// Intentionally `!Copy`, `!Clone` (clone discouraged for secrets).
pub struct SecureBuffer {
    data: Vec<u8>,
}

impl SecureBuffer {
    pub fn new(data: Vec<u8>) -> Self {
        Self { data }
    }

    pub fn from_slice(data: &[u8]) -> Self {
        Self {
            data: data.to_vec(),
        }
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.data
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}

impl Drop for SecureBuffer {
    fn drop(&mut self) {
        self.data.zeroize();
    }
}

// Intentionally NOT implementing: Clone, Debug, Display, Serialize.
// Secrets must not leak through these traits.
