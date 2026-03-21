//! WASM plugin host runtime for FreeSynergy.
//!
//! # Features
//! - `wasm` — enables the wasmtime-based WASM runtime ([`PluginRuntime`], [`PluginHandle`])
//!
//! # WASM plugins (feature `wasm`)
//!
//! ```rust,ignore
//! use fs_plugin_runtime::PluginRuntime;
//!
//! let runtime = PluginRuntime::new()?;
//! let mut handle = runtime.load_file(Path::new("zentinel.wasm"))?;
//! let response = handle.execute(&ctx)?;
//! ```
//!
//! # Process plugins (always available)
//!
//! Use [`ProcessPluginRunner`] for native executable plugins (no `wasm` feature required).

pub mod process_runner;

#[cfg(feature = "wasm")]
pub mod handle;
#[cfg(feature = "wasm")]
pub mod runtime;

pub use fs_plugin_sdk::{PluginContext, PluginManifest, PluginResponse};
pub use process_runner::ProcessPluginRunner;

#[cfg(feature = "wasm")]
pub use handle::PluginHandle;
#[cfg(feature = "wasm")]
pub use runtime::{PluginRuntime, PluginSandbox};
