// signing.rs — Signature verification gate for package installation.
//
// Before installing any package, the installer checks:
//   1. Is a signature present? → verify it.
//   2. No signature + trust_unsigned flag? → warn and proceed.
//   3. No signature + no flag? → reject.
//
// Pattern: Strategy — the verification algorithm is a `SignatureStrategy`
// trait object, making the choice between Ed25519 and no-op explicit rather
// than hidden behind `#[cfg(feature)]` inside a single method body.

use fs_error::FsError;

// ── SignaturePolicy ───────────────────────────────────────────────────────────

/// Controls how unsigned packages are handled.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SignaturePolicy {
    /// Require a valid signature for all packages.
    #[default]
    RequireSigned,
    /// Accept unsigned packages with a warning (--trust-unsigned).
    TrustUnsigned,
}

// ── VerifyOutcome ─────────────────────────────────────────────────────────────

/// Verification result for a package signature check.
#[derive(Debug, Clone)]
pub enum VerifyOutcome {
    /// Signature present and valid.
    Valid,
    /// No signature, but `TrustUnsigned` policy allows it (with a warning).
    UnsignedTrusted,
}

// ── SignatureStrategy (trait) ─────────────────────────────────────────────────

/// Verification algorithm used by [`SignatureVerifier`].
///
/// Two built-in implementations:
/// - [`Ed25519Strategy`] — real Ed25519 verification (requires feature `signing`)
/// - [`TrustAllStrategy`] — no-op, trusts everything (development / unsigned builds)
pub trait SignatureStrategy: Send + Sync {
    /// Verify `data` against an optional hex-encoded signature.
    fn verify(
        &self,
        data: &[u8],
        signature_hex: Option<&str>,
    ) -> Result<VerifyOutcome, FsError>;
}

// ── TrustAllStrategy ─────────────────────────────────────────────────────────

/// No-op strategy — trusts all packages, signed or not.
///
/// Use for development builds, integration tests, or any context where
/// signature enforcement is deliberately disabled.
pub struct TrustAllStrategy;

impl SignatureStrategy for TrustAllStrategy {
    fn verify(
        &self,
        _data: &[u8],
        _signature_hex: Option<&str>,
    ) -> Result<VerifyOutcome, FsError> {
        Ok(VerifyOutcome::UnsignedTrusted)
    }
}

// ── Ed25519Strategy ───────────────────────────────────────────────────────────

/// Ed25519 signature verification backed by `fs-crypto`.
///
/// Requires the `signing` feature on `fs-pkg`.
#[cfg(feature = "signing")]
pub struct Ed25519Strategy {
    /// Hex-encoded Ed25519 verifying key from `store.toml`.
    pub verifying_key_hex: String,
    /// How to handle unsigned packages.
    pub policy: SignaturePolicy,
}

#[cfg(feature = "signing")]
impl Ed25519Strategy {
    /// Create a strategy with the given public key and policy.
    pub fn new(verifying_key_hex: impl Into<String>, policy: SignaturePolicy) -> Self {
        Self { verifying_key_hex: verifying_key_hex.into(), policy }
    }
}

#[cfg(feature = "signing")]
impl SignatureStrategy for Ed25519Strategy {
    fn verify(
        &self,
        data: &[u8],
        signature_hex: Option<&str>,
    ) -> Result<VerifyOutcome, FsError> {
        match signature_hex {
            Some(sig_hex) => {
                let vk  = fs_crypto::FsVerifyingKey::from_hex(&self.verifying_key_hex)?;
                let sig = fs_crypto::PackageSignature::from_hex(sig_hex)?;
                vk.verify(data, &sig)?;
                Ok(VerifyOutcome::Valid)
            }
            None => match self.policy {
                SignaturePolicy::TrustUnsigned => {
                    eprintln!("WARNING: package has no signature (--trust-unsigned active)");
                    Ok(VerifyOutcome::UnsignedTrusted)
                }
                SignaturePolicy::RequireSigned => Err(FsError::internal(
                    "auth: package has no signature; use --trust-unsigned to override",
                )),
            },
        }
    }
}

// ── SignatureVerifier ─────────────────────────────────────────────────────────

/// Verifies package signatures using a pluggable [`SignatureStrategy`].
///
/// Construct via [`SignatureVerifier::ed25519`] (feature `signing`) or
/// [`SignatureVerifier::trust_all`] (always available).
///
/// # Example
///
/// ```rust,ignore
/// use fs_pkg::signing::{SignatureVerifier, SignaturePolicy};
///
/// // Production: real Ed25519 verification
/// let verifier = SignatureVerifier::ed25519("abc123...", SignaturePolicy::RequireSigned);
///
/// // Development: trust everything
/// let verifier = SignatureVerifier::trust_all();
///
/// let outcome = verifier.verify(&manifest_bytes, Some(&sig_hex))?;
/// ```
pub struct SignatureVerifier {
    strategy: Box<dyn SignatureStrategy>,
}

impl SignatureVerifier {
    /// Create a verifier backed by [`TrustAllStrategy`].
    ///
    /// Suitable for development builds or integration tests.
    pub fn trust_all() -> Self {
        Self { strategy: Box::new(TrustAllStrategy) }
    }

    /// Create a verifier backed by [`Ed25519Strategy`].
    ///
    /// Requires feature `signing`.
    #[cfg(feature = "signing")]
    pub fn ed25519(verifying_key_hex: impl Into<String>, policy: SignaturePolicy) -> Self {
        Self { strategy: Box::new(Ed25519Strategy::new(verifying_key_hex, policy)) }
    }

    /// Verify `data` against an optional hex-encoded signature.
    ///
    /// Delegates to the strategy selected at construction time.
    pub fn verify(
        &self,
        data: &[u8],
        signature_hex: Option<&str>,
    ) -> Result<VerifyOutcome, FsError> {
        self.strategy.verify(data, signature_hex)
    }
}
