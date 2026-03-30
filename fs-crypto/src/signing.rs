#![deny(clippy::all, clippy::pedantic, warnings)]
// signing.rs — Ed25519 package signing and verification for FreeSynergy.
//
// The store uses Ed25519 signatures to ensure package integrity and authenticity.
// Every package in the official store is signed by the FreeSynergy project key.
// Third-party packages may be unsigned; use `--trust-unsigned` at your own risk.
//
// Chain of trust:
//   [Binary] pinned verifying key (compiled in)
//     → [store.toml] verified against pinned key
//       → [catalog.toml] SHA-256 verified against store.toml
//         → [manifest.toml + files] SHA-256 verified against catalog
//           → [signature] Ed25519 verified before install
//
// Design:
//   FsSigningKey   — wraps ed25519_dalek::SigningKey (private, for signing)
//   FsVerifyingKey — wraps ed25519_dalek::VerifyingKey (public, for verification)
//   PackageSignature — detached signature over package bytes

use ed25519_dalek::{Signer, SigningKey, Verifier, VerifyingKey};
use rand_core::OsRng;
use sha2::{Digest, Sha256};

use fs_error::FsError;

use crate::provider::CryptoProvider;

// ── FsSigningKey ─────────────────────────────────────────────────────────────

/// Ed25519 private key — used to sign packages.
///
/// Keep this secret. Never distribute.
pub struct FsSigningKey {
    inner: SigningKey,
}

impl FsSigningKey {
    /// Generate a new random signing key.
    #[must_use]
    pub fn generate() -> Self {
        Self {
            inner: SigningKey::generate(&mut OsRng),
        }
    }

    /// Load a signing key from raw 32-byte seed.
    #[must_use]
    pub fn from_bytes(bytes: &[u8; 32]) -> Self {
        Self {
            inner: SigningKey::from_bytes(bytes),
        }
    }

    /// Load from a hex-encoded string.
    ///
    /// # Errors
    /// Returns `FsError::Parse` if the hex string is invalid or not 32 bytes.
    pub fn from_hex(s: &str) -> Result<Self, FsError> {
        let bytes =
            hex::decode(s).map_err(|e| FsError::parse(format!("signing key hex decode: {e}")))?;
        let arr: [u8; 32] = bytes
            .try_into()
            .map_err(|_| FsError::parse("signing key must be 32 bytes"))?;
        Ok(Self::from_bytes(&arr))
    }

    /// Encode the private key as a hex string (for storage in a secret file).
    #[must_use]
    pub fn to_hex(&self) -> String {
        hex::encode(self.inner.to_bytes())
    }

    /// The corresponding public verifying key.
    #[must_use]
    pub fn verifying_key(&self) -> FsVerifyingKey {
        FsVerifyingKey {
            inner: self.inner.verifying_key(),
        }
    }

    /// Sign `data` and return a detached [`PackageSignature`].
    ///
    /// Internally signs `SHA-256(data)` to keep the signature independent of data length.
    #[must_use]
    pub fn sign_package(&self, data: &[u8]) -> PackageSignature {
        let hash = Sha256::digest(data);
        let sig = self.inner.sign(&hash);
        PackageSignature {
            bytes: sig.to_bytes(),
        }
    }
}

impl CryptoProvider for FsSigningKey {
    fn sign(&self, data: &[u8]) -> Result<Vec<u8>, FsError> {
        Ok(self.sign_package(data).to_bytes().to_vec())
    }

    fn hash(&self, data: &[u8]) -> Result<Vec<u8>, FsError> {
        Ok(Sha256::digest(data).to_vec())
    }
}

// ── FsVerifyingKey ───────────────────────────────────────────────────────────

/// Ed25519 public key — embedded in the binary and in `store.toml`.
///
/// Used to verify package signatures before installation.
#[derive(Clone)]
pub struct FsVerifyingKey {
    inner: VerifyingKey,
}

impl FsVerifyingKey {
    /// Load from raw 32-byte compressed Edwards point.
    ///
    /// # Errors
    /// Returns `FsError::Parse` if the bytes are not a valid Ed25519 public key.
    pub fn from_bytes(bytes: &[u8; 32]) -> Result<Self, FsError> {
        VerifyingKey::from_bytes(bytes)
            .map(|inner| Self { inner })
            .map_err(|e| FsError::parse(format!("verifying key invalid: {e}")))
    }

    /// Load from a hex-encoded string.
    ///
    /// # Errors
    /// Returns `FsError::Parse` if the hex is invalid or not 32 bytes.
    pub fn from_hex(s: &str) -> Result<Self, FsError> {
        let bytes =
            hex::decode(s).map_err(|e| FsError::parse(format!("verifying key hex decode: {e}")))?;
        let arr: [u8; 32] = bytes
            .try_into()
            .map_err(|_| FsError::parse("verifying key must be 32 bytes"))?;
        Self::from_bytes(&arr)
    }

    /// Encode the public key as a hex string (for `store.toml`).
    #[must_use]
    pub fn to_hex(&self) -> String {
        hex::encode(self.inner.to_bytes())
    }

    /// Verify `signature` over `data`. Returns `Ok(())` on success.
    ///
    /// # Errors
    /// Returns `FsError::Internal` if signature verification fails.
    pub fn verify_package(&self, data: &[u8], signature: &PackageSignature) -> Result<(), FsError> {
        let hash = Sha256::digest(data);
        let sig = ed25519_dalek::Signature::from_bytes(&signature.bytes);
        self.inner
            .verify(&hash, &sig)
            .map_err(|e| FsError::internal(format!("auth: signature verification failed: {e}")))
    }
}

impl CryptoProvider for FsVerifyingKey {
    fn verify(&self, data: &[u8], signature: &[u8]) -> Result<(), FsError> {
        let pkg_sig = PackageSignature::from_bytes(signature)?;
        self.verify_package(data, &pkg_sig)
    }

    fn hash(&self, data: &[u8]) -> Result<Vec<u8>, FsError> {
        Ok(Sha256::digest(data).to_vec())
    }
}

// ── PackageSignature ──────────────────────────────────────────────────────────

/// Detached Ed25519 signature over a package's canonical bytes.
#[derive(Debug, Clone)]
pub struct PackageSignature {
    bytes: [u8; 64],
}

impl PackageSignature {
    /// Return the raw 64-byte signature.
    #[must_use]
    pub fn to_bytes(&self) -> [u8; 64] {
        self.bytes
    }

    /// Load from raw 64 bytes.
    ///
    /// # Errors
    /// Returns `FsError::Parse` if the slice is not exactly 64 bytes.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, FsError> {
        let arr: [u8; 64] = bytes
            .try_into()
            .map_err(|_| FsError::parse("signature must be 64 bytes"))?;
        Ok(Self { bytes: arr })
    }

    /// Encode as a hex string (stored in `installed_packages.signature`).
    #[must_use]
    pub fn to_hex(&self) -> String {
        hex::encode(self.bytes)
    }

    /// Load from a hex-encoded string.
    ///
    /// # Errors
    /// Returns `FsError::Parse` if the hex is invalid or not 64 bytes.
    pub fn from_hex(s: &str) -> Result<Self, FsError> {
        let bytes =
            hex::decode(s).map_err(|e| FsError::parse(format!("signature hex decode: {e}")))?;
        Self::from_bytes(&bytes)
    }
}

// ── keygen helper ─────────────────────────────────────────────────────────────

/// Generate a new Ed25519 keypair and return `(signing_key_hex, verifying_key_hex)`.
///
/// Used by `fsn store keygen`.
#[must_use]
pub fn generate_keypair() -> (String, String) {
    let sk = FsSigningKey::generate();
    let vk = sk.verifying_key();
    (sk.to_hex(), vk.to_hex())
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sign_verify_roundtrip() {
        let sk = FsSigningKey::generate();
        let vk = sk.verifying_key();
        let data = b"package-v1.0.0.tar.gz";
        let sig = sk.sign_package(data);
        assert!(vk.verify_package(data, &sig).is_ok());
    }

    #[test]
    fn wrong_key_fails_verification() {
        let sk1 = FsSigningKey::generate();
        let sk2 = FsSigningKey::generate();
        let sig = sk1.sign_package(b"data");
        let vk2 = sk2.verifying_key();
        assert!(vk2.verify_package(b"data", &sig).is_err());
    }

    #[test]
    fn tampered_data_fails_verification() {
        let sk = FsSigningKey::generate();
        let vk = sk.verifying_key();
        let sig = sk.sign_package(b"original");
        assert!(vk.verify_package(b"tampered", &sig).is_err());
    }

    #[test]
    fn signature_hex_roundtrip() {
        let sk = FsSigningKey::generate();
        let vk = sk.verifying_key();
        let data = b"test-data";
        let sig = sk.sign_package(data);
        let hex = sig.to_hex();
        let sig2 = PackageSignature::from_hex(&hex).unwrap();
        assert!(vk.verify_package(data, &sig2).is_ok());
    }

    #[test]
    fn key_hex_roundtrip() {
        let sk = FsSigningKey::generate();
        let vk = sk.verifying_key();

        let sk2 = FsSigningKey::from_hex(&sk.to_hex()).unwrap();
        let vk2 = FsVerifyingKey::from_hex(&vk.to_hex()).unwrap();

        let data = b"round-trip test";
        let sig = sk2.sign_package(data);
        assert!(vk2.verify_package(data, &sig).is_ok());
    }

    #[test]
    fn generate_keypair_returns_valid_hex() {
        let (sk_hex, vk_hex) = generate_keypair();
        assert_eq!(sk_hex.len(), 64); // 32 bytes → 64 hex chars
        assert_eq!(vk_hex.len(), 64);
        assert!(FsSigningKey::from_hex(&sk_hex).is_ok());
        assert!(FsVerifyingKey::from_hex(&vk_hex).is_ok());
    }

    #[test]
    fn crypto_provider_sign_verify_roundtrip() {
        let sk = FsSigningKey::generate();
        let vk = sk.verifying_key();
        let data = b"crypto-provider-test";
        let sig = sk.sign(data).unwrap();
        assert!(vk.verify(data, &sig).is_ok());
    }

    #[test]
    fn signature_from_bytes_wrong_length() {
        assert!(PackageSignature::from_bytes(&[0u8; 63]).is_err());
        assert!(PackageSignature::from_bytes(&[0u8; 65]).is_err());
        assert!(PackageSignature::from_bytes(&[0u8; 64]).is_ok());
    }
}
