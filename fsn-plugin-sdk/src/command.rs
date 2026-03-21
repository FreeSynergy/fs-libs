// PluginCommand trait — OOP interface for command handlers in plugin authors' code.

use crate::{PluginContext, PluginResponse};

/// A single command a plugin can handle.
///
/// Implement this trait for each command your plugin supports (e.g. `DeployCommand`,
/// `CleanCommand`). Register all commands with a [`CommandRouter`][crate::CommandRouter].
///
/// # Example
///
/// ```rust,ignore
/// struct DeployCommand;
///
/// impl PluginCommand for DeployCommand {
///     fn name(&self) -> &str { "deploy" }
///
///     fn execute(&self, ctx: &PluginContext) -> PluginResponse {
///         // Build response with files/commands...
///         PluginResponse::default()
///     }
/// }
/// ```
pub trait PluginCommand: Send + Sync {
    /// The command name this handler responds to (must match the `command` field in
    /// [`PluginContext`]).
    fn name(&self) -> &str;

    /// Execute the command using the provided context and return the response.
    fn execute(&self, ctx: &PluginContext) -> PluginResponse;
}
