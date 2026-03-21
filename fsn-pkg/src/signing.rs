// signing.rs — Signature verification gate for package installation.
//
// Before installing any package, the installer checks:
//   1. Is a signature present? → verify it.
//   2. No signature + trust_unsigned flag? → warn and proceed.
//   3. No signature + no flag? → reject.
//
// Pattern: Guard (verify_or_reject before install proceeds).

use fsn_error::FsnError;

/// Controls how unsigned packages are handled.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SignaturePolicy {
    /// Require a valid signature for all packages.
    #[default]
    RequireSigned,
    /// Accept unsigned packages with a warning (--trust-unsigned).
    TrustUnsigned,
}

/// Verification result for a package signature check.
#[derive(Debug, Clone)]
pub enum VerifyOutcome {
    /// Signature present and valid.
    Valid,
    /// No signature, but `TrustUnsigned` policy allows it (with a warning).
    UnsignedTrusted,
}

/// Verifies package signatures according to a [`SignaturePolicy`].
///
/// Requires the `signing` feature of `fsn-crypto`.
pub struct SignatureVerifier {
    /// Hex-encoded Ed25519 verifying key from `store.toml`.
    pub verifying_key_hex: String,
    /// How to handle unsigned packages.
    pub policy: SignaturePolicy,
}

impl SignatureVerifier {
    /// Create a verifier with the official store public key.
    pub fn new(verifying_key_hex: impl Into<String>, policy: SignaturePolicy) -> Self {
        Self {
            verifying_key_hex: verifying_key_hex.into(),
            policy,
        }
    }

    /// Verify `data` against an optional `signature_hex`.
    ///
    /// - If `signature_hex` is `Some`, verifies with the stored public key.
    /// - If `signature_hex` is `None` and policy is `TrustUnsigned`, returns `UnsignedTrusted`.
    /// - If `signature_hex` is `None` and policy is `RequireSigned`, returns an error.
    #[cfg(feature = "signing")]
    pub fn verify(
        &self,
        data: &[u8],
        signature_hex: Option<&str>,
    ) -> Result<VerifyOutcome, FsnError> {
        match signature_hex {
            Some(sig_hex) => {
                let vk = fsn_crypto::FsnVerifyingKey::from_hex(&self.verifying_key_hex)?;
                let sig = fsn_crypto::PackageSignature::from_hex(sig_hex)?;
                vk.verify(data, &sig)?;
                Ok(VerifyOutcome::Valid)
            }
            None => match self.policy {
                SignaturePolicy::TrustUnsigned => {
                    eprintln!("WARNING: package has no signature (--trust-unsigned active)");
                    Ok(VerifyOutcome::UnsignedTrusted)
                }
                SignaturePolicy::RequireSigned => {
                    Err(FsnError::internal(
                        "auth: package has no signature; use --trust-unsigned to override",
                    ))
                }
            },
        }
    }

    /// Version without the `signing` feature — always allows (for builds without crypto).
    #[cfg(not(feature = "signing"))]
    pub fn verify(
        &self,
        _data: &[u8],
        _signature_hex: Option<&str>,
    ) -> Result<VerifyOutcome, FsnError> {
        Ok(VerifyOutcome::UnsignedTrusted)
    }
}
