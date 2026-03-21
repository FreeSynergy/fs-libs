// fsn-bot/src/commands/status.rs — StatusCommand: reports system status.

use async_trait::async_trait;
use fsn_auth::Permission;

use crate::command::{BotCommand, CommandResult};
use crate::context::CommandContext;
use crate::error::BotError;

/// Reports system status. Requires the `bot:status` permission.
///
/// The default implementation reports `"All systems operational."`.
/// Replace with a real status check by implementing [`StatusProvider`].
pub struct StatusCommand {
    provider: Box<dyn StatusProvider>,
}

/// Provides the status text for [`StatusCommand`].
pub trait StatusProvider: Send + Sync {
    /// Return a human-readable status summary.
    fn status(&self) -> String;
}

/// Default provider — always reports everything is OK.
pub struct DefaultStatusProvider;

impl StatusProvider for DefaultStatusProvider {
    fn status(&self) -> String {
        "All systems operational.".to_string()
    }
}

impl StatusCommand {
    /// Create with a custom status provider.
    pub fn new(provider: Box<dyn StatusProvider>) -> Self {
        Self { provider }
    }

    /// Create with the default "all good" provider.
    pub fn default() -> Self {
        Self::new(Box::new(DefaultStatusProvider))
    }
}

#[async_trait]
impl BotCommand for StatusCommand {
    fn name(&self) -> &str {
        "status"
    }

    fn usage(&self) -> &str {
        "status — show system status"
    }

    fn required_permission(&self) -> Option<Permission> {
        Some(Permission::new("bot:status"))
    }

    async fn execute(&self, ctx: &CommandContext) -> Result<CommandResult, BotError> {
        let msg = self.provider.status();
        ctx.reply(&msg).await?;
        Ok(CommandResult::Replied(msg))
    }
}
