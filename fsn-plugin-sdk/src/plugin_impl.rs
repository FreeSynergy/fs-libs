// PluginImpl — OOP trait that WASM plugin authors implement.

use serde::{Deserialize, Serialize};

use crate::{PluginContext, PluginResponse};

// ── PluginManifest ────────────────────────────────────────────────────────────

/// Metadata returned by a WASM plugin at load time.
///
/// The host reads this once via `plugin_manifest_json()` to discover what
/// commands the plugin supports and to validate its identity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifest {
    /// Unique plugin identifier, e.g. `"zentinel"`.
    pub id: String,

    /// Semantic version string, e.g. `"0.1.0"`.
    pub version: String,

    /// Commands this plugin handles, e.g. `["deploy", "clean"]`.
    pub commands: Vec<String>,

    /// Human-readable description of the plugin.
    pub description: String,
}

// ── PluginImpl ────────────────────────────────────────────────────────────────

/// Trait that WASM plugin authors implement.
///
/// The [`plugin_main!`] macro generates WASM exports that delegate to this trait.
/// Plugin types must also implement [`Default`] so the macro can instantiate them.
///
/// # Example
///
/// ```rust,ignore
/// use fsn_plugin_sdk::{plugin_main, PluginContext, PluginImpl, PluginManifest, PluginResponse};
///
/// #[derive(Default)]
/// struct MyPlugin;
///
/// impl PluginImpl for MyPlugin {
///     fn manifest(&self) -> PluginManifest {
///         PluginManifest {
///             id: "my-plugin".into(),
///             version: "0.1.0".into(),
///             commands: vec!["deploy".into()],
///             description: "My plugin".into(),
///         }
///     }
///
///     fn execute(&self, command: &str, ctx: &PluginContext) -> Result<PluginResponse, String> {
///         match command {
///             "deploy" => Ok(PluginResponse::default()),
///             other    => Err(format!("unknown command: {other}")),
///         }
///     }
/// }
///
/// plugin_main!(MyPlugin);
/// ```
pub trait PluginImpl: Send + Sync {
    /// Return the plugin manifest.
    fn manifest(&self) -> PluginManifest;

    /// Execute a command.
    ///
    /// `command` is the name of the command to run (must be listed in the manifest).
    /// `ctx` is the fully populated [`PluginContext`] for this invocation.
    ///
    /// Return `Ok(response)` on success, `Err(message)` on failure.
    fn execute(&self, command: &str, ctx: &PluginContext) -> Result<PluginResponse, String>;
}
