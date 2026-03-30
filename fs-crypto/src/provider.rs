#![deny(clippy::all, clippy::pedantic, warnings)]

use fs_error::FsError;

/// Strategy trait for all `FreeSynergy` cryptographic operations.
///
/// Concrete providers implement only the operations they support.
/// Unsupported operations return `Err(FsError::Internal(...))` via the defaults.
/// Program against this trait — never against concrete crypto types directly.
///
/// # Object safety
///
/// This trait is object-safe and can be used as `Box<dyn CryptoProvider>`.
pub trait CryptoProvider {
    /// Encrypt `plaintext`. Returns ciphertext bytes.
    ///
    /// # Errors
    /// Returns `FsError::Internal` if encryption fails or is not supported.
    fn encrypt(&self, plaintext: &[u8]) -> Result<Vec<u8>, FsError> {
        let _ = plaintext;
        Err(FsError::internal("encrypt: not supported by this provider"))
    }

    /// Decrypt `ciphertext`. Returns plaintext bytes.
    ///
    /// # Errors
    /// Returns `FsError::Internal` if decryption fails or is not supported.
    fn decrypt(&self, ciphertext: &[u8]) -> Result<Vec<u8>, FsError> {
        let _ = ciphertext;
        Err(FsError::internal("decrypt: not supported by this provider"))
    }

    /// Sign `data`. Returns signature bytes (format is provider-specific).
    ///
    /// # Errors
    /// Returns `FsError::Internal` if signing fails or is not supported.
    fn sign(&self, data: &[u8]) -> Result<Vec<u8>, FsError> {
        let _ = data;
        Err(FsError::internal("sign: not supported by this provider"))
    }

    /// Verify `signature` over `data`. Returns `Ok(())` when valid.
    ///
    /// # Errors
    /// Returns `FsError::Auth` when verification fails, or `FsError::Internal`
    /// when the operation is not supported.
    fn verify(&self, data: &[u8], signature: &[u8]) -> Result<(), FsError> {
        let _ = (data, signature);
        Err(FsError::internal("verify: not supported by this provider"))
    }

    /// Hash `data`. Returns digest bytes (algorithm is provider-specific).
    ///
    /// # Errors
    /// Returns `FsError::Internal` if hashing fails or is not supported.
    fn hash(&self, data: &[u8]) -> Result<Vec<u8>, FsError> {
        let _ = data;
        Err(FsError::internal("hash: not supported by this provider"))
    }
}
