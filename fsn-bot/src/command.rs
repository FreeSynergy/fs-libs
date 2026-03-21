// fsn-bot/src/command.rs — BotCommand trait and CommandResult.

use async_trait::async_trait;
use fsn_auth::Permission;

use crate::context::CommandContext;
use crate::error::BotError;

// ── CommandResult ─────────────────────────────────────────────────────────────

/// The outcome of a command execution.
#[derive(Debug, Clone)]
pub enum CommandResult {
    /// Replied with `text` — already sent via `ctx.reply()`.
    Replied(String),
    /// The command produced no reply (e.g. fire-and-forget action).
    Silent,
}

// ── BotCommand ────────────────────────────────────────────────────────────────

/// A single bot command.
///
/// # Implementing a command
///
/// ```rust,ignore
/// use fsn_bot::{BotCommand, CommandContext, CommandResult, BotError};
/// use async_trait::async_trait;
///
/// pub struct PingCommand;
///
/// #[async_trait]
/// impl BotCommand for PingCommand {
///     fn name(&self) -> &str { "ping" }
///     fn usage(&self) -> &str { "ping — reply with pong" }
///     fn required_permission(&self) -> Option<fsn_auth::Permission> { None }
///
///     async fn execute(&self, ctx: &CommandContext) -> Result<CommandResult, BotError> {
///         ctx.reply("pong").await?;
///         Ok(CommandResult::Replied("pong".into()))
///     }
/// }
/// ```
#[async_trait]
pub trait BotCommand: Send + Sync {
    /// Command name without prefix, e.g. `"ping"`, `"deploy"`.
    fn name(&self) -> &str;

    /// One-line usage string shown in help output.
    fn usage(&self) -> &str;

    /// Permission required to run this command.
    ///
    /// `None` means no permission check — anyone can run it.
    fn required_permission(&self) -> Option<Permission>;

    /// Execute the command.
    async fn execute(&self, ctx: &CommandContext) -> Result<CommandResult, BotError>;
}
