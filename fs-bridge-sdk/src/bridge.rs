// Bridge trait and core types for FreeSynergy service connectors.

use serde::{Deserialize, Serialize};
use fs_error::FsError;

// ── BridgeInfo ────────────────────────────────────────────────────────────────

/// Metadata returned when a bridge is probed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeInfo {
    /// Unique service identifier, e.g. `"forgejo"`.
    pub service_id: String,
    /// Version string reported by the service.
    pub version: String,
    /// Base URL of the service endpoint.
    pub base_url: String,
    /// Whether the service is reachable and healthy.
    pub healthy: bool,
}

// ── BridgeConfig ──────────────────────────────────────────────────────────────

/// Configuration for an HTTP-based bridge.
#[derive(Debug, Clone)]
pub struct BridgeConfig {
    /// Unique service identifier, e.g. `"forgejo"`.
    pub service_id: String,
    /// Base URL of the service endpoint (no trailing slash).
    pub base_url: String,
    /// Optional Bearer token for authentication.
    pub token: Option<String>,
    /// Request timeout in seconds (default: 10).
    pub timeout_secs: u64,
}

impl BridgeConfig {
    /// Create a minimal config with defaults (no token, 10 s timeout).
    pub fn new(service_id: impl Into<String>, base_url: impl Into<String>) -> Self {
        Self {
            service_id: service_id.into(),
            base_url: base_url.into(),
            token: None,
            timeout_secs: 10,
        }
    }

    /// Attach a Bearer token to this config.
    pub fn with_token(mut self, token: impl Into<String>) -> Self {
        self.token = Some(token.into());
        self
    }

    /// Override the request timeout.
    pub fn with_timeout(mut self, secs: u64) -> Self {
        self.timeout_secs = secs;
        self
    }
}

// ── Bridge trait ──────────────────────────────────────────────────────────────

/// A connector to an external service.
///
/// Implement this trait for each service type (git, mail, iam, etc.).
/// The `probe` method uses RPITIT (Return Position Impl Trait in Trait) and
/// is therefore not object-safe on its own. Use [`ProbableBridge`] for
/// trait-object scenarios.
pub trait Bridge: Send + Sync {
    /// Unique service identifier, e.g. `"forgejo"`.
    fn service_id(&self) -> &str;

    /// The base URL of this bridge's target.
    fn base_url(&self) -> &str;

    /// Test the connection and return service metadata.
    fn probe(&self) -> impl std::future::Future<Output = Result<BridgeInfo, FsError>> + Send;
}

// ── ProbableBridge (object-safe) ──────────────────────────────────────────────

/// Object-safe version of [`Bridge`] using boxed futures.
///
/// Use this when you need `dyn ProbableBridge` (e.g. in [`super::registry::BridgeRegistry`]).
/// Implement `Bridge` for your concrete type, then blanket-impl `ProbableBridge` automatically
/// via the provided blanket impl, or implement it manually.
pub trait ProbableBridge: Send + Sync {
    /// Unique service identifier, e.g. `"forgejo"`.
    fn service_id(&self) -> &str;

    /// The base URL of this bridge's target.
    fn base_url(&self) -> &str;

    /// Test the connection and return service metadata (boxed future for object safety).
    fn probe(
        &self,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<BridgeInfo, FsError>> + Send + '_>>;
}
