//! WASM plugin SDK for FreeSynergy.
//!
//! This crate is used by **plugin authors** who compile their plugin to WASM.
//! It provides:
//!
//! - Protocol types: [`PluginContext`], [`PluginResponse`], [`ModuleManifest`]
//! - WASM OOP interface: [`PluginImpl`] trait + [`PluginManifest`]
//! - Entry point: [`plugin_main!`] macro (generates WASM exports)
//! - Process fallback: [`PluginCommand`] trait + [`CommandRouter`] + [`run_plugin`]
//!
//! # Quick start (WASM plugin)
//!
//! ```rust,ignore
//! use fsn_plugin_sdk::{plugin_main, PluginContext, PluginImpl, PluginManifest, PluginResponse};
//!
//! #[derive(Default)]
//! struct ZentinelPlugin;
//!
//! impl PluginImpl for ZentinelPlugin {
//!     fn manifest(&self) -> PluginManifest {
//!         PluginManifest {
//!             id:          "zentinel".into(),
//!             version:     "0.1.0".into(),
//!             commands:    vec!["deploy".into(), "clean".into()],
//!             description: "Zentinel reverse proxy plugin".into(),
//!         }
//!     }
//!
//!     fn execute(&self, command: &str, ctx: &PluginContext) -> Result<PluginResponse, String> {
//!         match command {
//!             "deploy" => Ok(PluginResponse::default()),
//!             "clean"  => Ok(PluginResponse::default()),
//!             other    => Err(format!("unknown command: {other}")),
//!         }
//!     }
//! }
//!
//! plugin_main!(ZentinelPlugin);
//! ```

pub mod command;
pub mod context;
pub mod entry;
pub mod macros;
pub mod manifest;
pub mod plugin_impl;
pub mod response;
pub mod router;
pub mod traits;

// WASM plugin interface
pub use plugin_impl::{PluginImpl, PluginManifest};

// Protocol types
pub use command::PluginCommand;
pub use context::{InstanceInfo, PeerRoute, PeerService, PluginContext};
pub use entry::run_plugin;
pub use manifest::{ManifestInputs, ManifestOutputFile, ModuleManifest};
pub use response::{LogLevel, LogLine, OutputFile, PluginResponse, ShellCommand};
pub use router::CommandRouter;
