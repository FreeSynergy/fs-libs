#![deny(clippy::all, clippy::pedantic, warnings)]
//! Cryptographic primitives for `FreeSynergy`.
//!
//! # Features
//! - `age` — age X25519 encryption/decryption + keypair generation
//! - `mtls` — mTLS certificate generation via rcgen
//! - `keygen` — random secret generation and key derivation (PBKDF2-HMAC-SHA256)
//! - `signing` — Ed25519 package signing and verification
//! - `tokens` — join-token and recovery-token generation
//! - `hmac` — HMAC-SHA256 provider for invite tokens and message authentication

#[cfg(any(feature = "keygen", feature = "tokens"))]
pub(crate) mod base64url;
pub mod provider;

#[cfg(feature = "age")]
pub mod age_crypto;
#[cfg(feature = "hmac")]
pub mod hmac_provider;
#[cfg(feature = "keygen")]
pub mod keygen;
#[cfg(feature = "mtls")]
pub mod mtls;
#[cfg(feature = "signing")]
pub mod signing;
#[cfg(feature = "tokens")]
pub mod tokens;

// ── Flat re-exports ───────────────────────────────────────────────────────────

pub use provider::CryptoProvider;

#[cfg(feature = "age")]
pub use age_crypto::{
    generate_age_keypair, AgeDecryptor, AgeEncryptor, AgePassphraseDecryptor,
    AgePassphraseEncryptor,
};

#[cfg(feature = "hmac")]
pub use hmac_provider::HmacProvider;

#[cfg(feature = "keygen")]
pub use keygen::{derive_key, random_bytes, random_hex, random_secret};

#[cfg(feature = "mtls")]
pub use mtls::{CaBundle, CertBundle};

#[cfg(feature = "signing")]
pub use signing::{generate_keypair, FsSigningKey, FsVerifyingKey, PackageSignature};

#[cfg(feature = "tokens")]
pub use tokens::{generate_recovery_token, JoinToken, JoinTokenError};
