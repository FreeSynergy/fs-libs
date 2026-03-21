//! Bridge interface traits and base types for FreeSynergy service connectors.
//!
//! A *bridge* is a typed connector to an external service API such as
//! Forgejo (git), Stalwart (mail), Kanidm (IAM), or any other service that
//! FreeSynergy.Node manages.
//!
//! This crate provides the **interface layer** only — concrete bridge
//! implementations live in consuming projects and link against this crate.
//!
//! # Key Types
//!
//! | Type | Purpose |
//! |------|---------|
//! | [`Bridge`] | RPITIT trait for a service connector (non-object-safe) |
//! | [`ProbableBridge`] | Object-safe version using boxed futures |
//! | [`BridgeConfig`] | HTTP configuration (URL, token, timeout) |
//! | [`BridgeInfo`] | Metadata returned by a probe |
//! | [`HttpBridge`] | Composable HTTP base for concrete bridges |
//! | [`BridgeRegistry`] | Service-locator for registered bridges |
//!
//! # Quick Start
//!
//! ```rust,ignore
//! use fs_bridge_sdk::{BridgeConfig, HttpBridge, BridgeInfo, ProbableBridge};
//! use fs_error::FsError;
//!
//! struct ForgejooBridge { http: HttpBridge }
//!
//! impl ProbableBridge for ForgejooBridge {
//!     fn service_id(&self) -> &str { "forgejo" }
//!     fn base_url(&self)   -> &str { self.http.config().base_url.as_str() }
//!
//!     fn probe(&self) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<BridgeInfo, FsError>> + Send + '_>> {
//!         Box::pin(async move {
//!             let info: serde_json::Value = self.http.get("api/v1/version").await?;
//!             Ok(BridgeInfo {
//!                 service_id: "forgejo".into(),
//!                 version: info["version"].as_str().unwrap_or("?").to_string(),
//!                 base_url: self.base_url().to_string(),
//!                 healthy: true,
//!             })
//!         })
//!     }
//! }
//! ```

pub mod bridge;
pub mod http;
pub mod registry;

pub use bridge::{Bridge, BridgeConfig, BridgeInfo, ProbableBridge};
pub use http::HttpBridge;
pub use registry::BridgeRegistry;
