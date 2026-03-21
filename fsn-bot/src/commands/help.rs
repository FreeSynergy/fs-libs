// fsn-bot/src/commands/help.rs — HelpCommand: lists all registered commands.

use async_trait::async_trait;
use fsn_auth::Permission;

use crate::command::{BotCommand, CommandResult};
use crate::context::CommandContext;
use crate::error::BotError;
use crate::registry::CommandRegistry;

use std::sync::Arc;

/// Lists all registered commands and their usage strings.
pub struct HelpCommand {
    registry: Arc<CommandRegistry>,
    prefix: String,
}

impl HelpCommand {
    /// Create a help command that reads from the given registry.
    pub fn new(registry: Arc<CommandRegistry>, prefix: impl Into<String>) -> Self {
        Self { registry, prefix: prefix.into() }
    }
}

#[async_trait]
impl BotCommand for HelpCommand {
    fn name(&self) -> &str {
        "help"
    }

    fn usage(&self) -> &str {
        "help — list all available commands"
    }

    fn required_permission(&self) -> Option<Permission> {
        None
    }

    async fn execute(&self, ctx: &CommandContext) -> Result<CommandResult, BotError> {
        let mut lines = vec!["**Available commands:**".to_string()];

        for (_, cmd) in self.registry.all() {
            lines.push(format!("`{}{}` — {}", self.prefix, cmd.name(), cmd.usage()));
        }

        let reply = lines.join("\n");
        ctx.reply_markdown(&reply).await?;
        Ok(CommandResult::Replied(reply))
    }
}
