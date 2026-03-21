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
//   FsnSigningKey   — wraps ed25519_dalek::SigningKey (private, for signing)
//   FsnVerifyingKey — wraps ed25519_dalek::VerifyingKey (public, for verification)
//   PackageSignature — detached signature over package bytes

use ed25519_dalek::{Signer, SigningKey, Verifier, VerifyingKey};
use hex;
use rand::rngs::OsRng;
use sha2::{Digest, Sha256};

use fsn_error::FsnError;

// ── FsnSigningKey ─────────────────────────────────────────────────────────────

/// Ed25519 private key — used to sign packages.
///
/// Keep this secret. Never distribute.
pub struct FsnSigningKey {
    inner: SigningKey,
}

impl FsnSigningKey {
    /// Generate a new random signing key.
    pub fn generate() -> Self {
        Self { inner: SigningKey::generate(&mut OsRng) }
    }

    /// Load a signing key from raw 32-byte seed.
    pub fn from_bytes(bytes: &[u8; 32]) -> Self {
        Self { inner: SigningKey::from_bytes(bytes) }
    }

    /// Load from a hex-encoded string.
    pub fn from_hex(s: &str) -> Result<Self, FsnError> {
        let bytes = hex::decode(s)
            .map_err(|e| FsnError::parse(format!("signing key hex decode: {e}")))?;
        let arr: [u8; 32] = bytes
            .try_into()
            .map_err(|_| FsnError::parse("signing key must be 32 bytes"))?;
        Ok(Self::from_bytes(&arr))
    }

    /// Encode the private key as a hex string (for storage in a secret file).
    pub fn to_hex(&self) -> String {
        hex::encode(self.inner.to_bytes())
    }

    /// The corresponding public verifying key.
    pub fn verifying_key(&self) -> FsnVerifyingKey {
        FsnVerifyingKey { inner: self.inner.verifying_key() }
    }

    /// Sign `data` and return a detached [`PackageSignature`].
    ///
    /// Internally signs SHA-256(data) to keep the signature independent of data length.
    pub fn sign(&self, data: &[u8]) -> PackageSignature {
        let hash = Sha256::digest(data);
        let sig = self.inner.sign(&hash);
        PackageSignature { bytes: sig.to_bytes() }
    }
}

// ── FsnVerifyingKey ───────────────────────────────────────────────────────────

/// Ed25519 public key — embedded in the binary and in `store.toml`.
///
/// Used to verify package signatures before installation.
#[derive(Clone)]
pub struct FsnVerifyingKey {
    inner: VerifyingKey,
}

impl FsnVerifyingKey {
    /// Load from raw 32-byte compressed Edwards point.
    pub fn from_bytes(bytes: &[u8; 32]) -> Result<Self, FsnError> {
        VerifyingKey::from_bytes(bytes)
            .map(|inner| Self { inner })
            .map_err(|e| FsnError::parse(format!("verifying key invalid: {e}")))
    }

    /// Load from a hex-encoded string.
    pub fn from_hex(s: &str) -> Result<Self, FsnError> {
        let bytes = hex::decode(s)
            .map_err(|e| FsnError::parse(format!("verifying key hex decode: {e}")))?;
        let arr: [u8; 32] = bytes
            .try_into()
            .map_err(|_| FsnError::parse("verifying key must be 32 bytes"))?;
        Self::from_bytes(&arr)
    }

    /// Encode the public key as a hex string (for `store.toml`).
    pub fn to_hex(&self) -> String {
        hex::encode(self.inner.to_bytes())
    }

    /// Verify `signature` over `data`. Returns `Ok(())` on success.
    pub fn verify(&self, data: &[u8], signature: &PackageSignature) -> Result<(), FsnError> {
        let hash = Sha256::digest(data);
        let sig = ed25519_dalek::Signature::from_bytes(&signature.bytes);
        self.inner
            .verify(&hash, &sig)
            .map_err(|e| FsnError::internal(format!("auth: signature verification failed: {e}")))
    }
}

// ── PackageSignature ──────────────────────────────────────────────────────────

/// Detached Ed25519 signature over a package's canonical bytes.
#[derive(Debug, Clone)]
pub struct PackageSignature {
    bytes: [u8; 64],
}

impl PackageSignature {
    /// Encode as a hex string (stored in `installed_packages.signature`).
    pub fn to_hex(&self) -> String {
        hex::encode(self.bytes)
    }

    /// Load from a hex-encoded string.
    pub fn from_hex(s: &str) -> Result<Self, FsnError> {
        let bytes = hex::decode(s)
            .map_err(|e| FsnError::parse(format!("signature hex decode: {e}")))?;
        let arr: [u8; 64] = bytes
            .try_into()
            .map_err(|_| FsnError::parse("signature must be 64 bytes"))?;
        Ok(Self { bytes: arr })
    }
}

// ── keygen helper ─────────────────────────────────────────────────────────────

/// Generate a new Ed25519 keypair and return `(signing_key_hex, verifying_key_hex)`.
///
/// Used by `fsn store keygen`.
pub fn generate_keypair() -> (String, String) {
    let sk = FsnSigningKey::generate();
    let vk = sk.verifying_key();
    (sk.to_hex(), vk.to_hex())
}
