// fsn-bot/src/commands/ping.rs — PingCommand: replies with "pong".

use async_trait::async_trait;
use fsn_auth::Permission;

use crate::command::{BotCommand, CommandResult};
use crate::context::CommandContext;
use crate::error::BotError;

/// Responds to `ping` with `pong` — useful for checking the bot is alive.
pub struct PingCommand;

#[async_trait]
impl BotCommand for PingCommand {
    fn name(&self) -> &str {
        "ping"
    }

    fn usage(&self) -> &str {
        "ping — check if the bot is alive"
    }

    fn required_permission(&self) -> Option<Permission> {
        None
    }

    async fn execute(&self, ctx: &CommandContext) -> Result<CommandResult, BotError> {
        ctx.reply("pong").await?;
        Ok(CommandResult::Replied("pong".into()))
    }
}
