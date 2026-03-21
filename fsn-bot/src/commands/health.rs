// fsn-bot/src/commands/health.rs — HealthQueryCommand: reports health of a service/project.

use async_trait::async_trait;
use fsn_auth::Permission;

use crate::command::{BotCommand, CommandResult};
use crate::context::CommandContext;
use crate::error::BotError;

/// Reports health status for a named service or project.
///
/// Usage: `!health [<service>]`
///
/// Requires the `node:read` permission.
///
/// The default implementation delegates to a [`HealthQueryProvider`].
/// Wire it up to `fsn-health` for real data.
pub struct HealthQueryCommand {
    provider: Box<dyn HealthQueryProvider>,
}

/// Provides health information for a service or project.
pub trait HealthQueryProvider: Send + Sync {
    /// Return a human-readable health summary for `target`.
    ///
    /// `target` is `None` when the user ran `!health` without arguments.
    fn query(&self, target: Option<&str>) -> String;
}

/// Stub provider — always reports healthy.
pub struct StubHealthProvider;

impl HealthQueryProvider for StubHealthProvider {
    fn query(&self, target: Option<&str>) -> String {
        match target {
            Some(t) => format!("✓ {t}: healthy"),
            None    => "✓ All services healthy.".to_string(),
        }
    }
}

impl HealthQueryCommand {
    /// Create with a custom health provider.
    pub fn new(provider: Box<dyn HealthQueryProvider>) -> Self {
        Self { provider }
    }

    /// Create with the stub provider (always reports healthy).
    pub fn stub() -> Self {
        Self::new(Box::new(StubHealthProvider))
    }
}

#[async_trait]
impl BotCommand for HealthQueryCommand {
    fn name(&self) -> &str {
        "health"
    }

    fn usage(&self) -> &str {
        "health [<service>] — show service health"
    }

    fn required_permission(&self) -> Option<Permission> {
        Some(Permission::new("node:read"))
    }

    async fn execute(&self, ctx: &CommandContext) -> Result<CommandResult, BotError> {
        let target = ctx.args.first().map(String::as_str);
        let reply = self.provider.query(target);
        ctx.reply(&reply).await?;
        Ok(CommandResult::Replied(reply))
    }
}
