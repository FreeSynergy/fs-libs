//! Cryptographic primitives for FreeSynergy.
//!
//! # Features
//! - `age` — age X25519 encryption/decryption + keypair generation
//! - `mtls` — mTLS certificate generation via rcgen
//! - `keygen` — random secret generation and key derivation
//! - `signing` — Ed25519 package signing and verification

#[cfg(feature = "age")]
pub mod age_crypto;
#[cfg(feature = "mtls")]
pub mod mtls;
#[cfg(feature = "keygen")]
pub mod keygen;
#[cfg(feature = "tokens")]
pub mod tokens;
#[cfg(feature = "signing")]
pub mod signing;

#[cfg(feature = "age")]
pub use age_crypto::{
    generate_age_keypair, AgeDecryptor, AgeEncryptor,
    AgePassphraseDecryptor, AgePassphraseEncryptor,
};
#[cfg(feature = "mtls")]
pub use mtls::{CaBundle, CertBundle};
#[cfg(feature = "keygen")]
pub use keygen::{derive_key, random_bytes, random_hex, random_secret};
#[cfg(feature = "tokens")]
pub use tokens::{generate_recovery_token, JoinToken, JoinTokenError};
#[cfg(feature = "signing")]
pub use signing::{generate_keypair, FsnSigningKey, FsnVerifyingKey, PackageSignature};
