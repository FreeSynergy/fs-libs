// CommandRouter — dispatches PluginContext to the matching PluginCommand.

use crate::{PluginCommand, PluginContext, PluginResponse};

/// Routes an incoming [`PluginContext`] to the correct [`PluginCommand`] handler.
///
/// Build a router in your plugin's `main`, register all supported commands,
/// then pass it to [`run_plugin`][crate::run_plugin]:
///
/// ```rust,ignore
/// let mut router = CommandRouter::new();
/// router.register(DeployCommand);
/// router.register(CleanCommand);
/// run_plugin(&router);
/// ```
pub struct CommandRouter {
    commands: Vec<Box<dyn PluginCommand>>,
}

impl CommandRouter {
    /// Create an empty router with no registered commands.
    pub fn new() -> Self {
        Self {
            commands: Vec::new(),
        }
    }

    /// Register a command handler.
    ///
    /// The handler's [`PluginCommand::name`] must be unique within this router.
    pub fn register(&mut self, command: impl PluginCommand + 'static) {
        self.commands.push(Box::new(command));
    }

    /// Dispatch the context to the matching command handler.
    ///
    /// Returns the handler's response, or an error response if no handler matches
    /// `ctx.command`.
    pub fn dispatch(&self, ctx: &PluginContext) -> PluginResponse {
        match self.commands.iter().find(|c| c.name() == ctx.command) {
            Some(handler) => handler.execute(ctx),
            None => PluginResponse::err(format!(
                "unknown command: {}; supported: [{}]",
                ctx.command,
                self.commands
                    .iter()
                    .map(|c| c.name())
                    .collect::<Vec<_>>()
                    .join(", ")
            )),
        }
    }
}

impl Default for CommandRouter {
    fn default() -> Self {
        Self::new()
    }
}
