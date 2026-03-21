// fsn-bot/src/commands/deploy.rs — DeployCommand: triggers a deploy event on fsn-bus.

use std::sync::Arc;

use async_trait::async_trait;
use fsn_auth::Permission;
use fsn_bus::{Event, Router};
use tokio::sync::Mutex;
use tracing::info;

use crate::command::{BotCommand, CommandResult};
use crate::context::CommandContext;
use crate::error::BotError;

/// Triggers a `deploy.requested` event on the bus.
///
/// Usage: `!deploy <service> [<host>]`
///
/// Requires the `node:deploy` permission.
pub struct DeployCommand {
    bus: Arc<Mutex<Router>>,
    source: String,
}

impl DeployCommand {
    /// Create with a reference to the bus router.
    ///
    /// `source` is the component name reported in the event metadata,
    /// e.g. `"fsn-bot"`.
    pub fn new(bus: Arc<Mutex<Router>>, source: impl Into<String>) -> Self {
        Self { bus, source: source.into() }
    }
}

#[async_trait]
impl BotCommand for DeployCommand {
    fn name(&self) -> &str {
        "deploy"
    }

    fn usage(&self) -> &str {
        "deploy <service> [<host>] — trigger a deployment"
    }

    fn required_permission(&self) -> Option<Permission> {
        Some(Permission::new("node:deploy"))
    }

    async fn execute(&self, ctx: &CommandContext) -> Result<CommandResult, BotError> {
        let service = ctx
            .args
            .first()
            .ok_or_else(|| BotError::invalid_args("deploy", "missing service name"))?
            .clone();

        let host = ctx.args.get(1).cloned();

        let payload = serde_json::json!({
            "service": service,
            "host": host,
            "requested_by": ctx.sender,
        });

        let event = Event::new("deploy.requested", &self.source, payload)
            .map_err(|e| BotError::internal(e.to_string()))?;

        self.bus.lock().await.dispatch(&event).await;

        info!(service = %service, sender = %ctx.sender, "deploy requested via bot");

        let reply = match &host {
            Some(h) => format!("Deploy of **{service}** on **{h}** requested."),
            None    => format!("Deploy of **{service}** requested."),
        };
        ctx.reply_markdown(&reply).await?;
        Ok(CommandResult::Replied(reply))
    }
}
