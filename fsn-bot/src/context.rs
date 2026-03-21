// fsn-bot/src/context.rs — CommandContext passed to every BotCommand.

use std::sync::Arc;

use fsn_auth::{Permission, PermissionSet};
use fsn_channel::{Channel, ChannelMessage};

use crate::error::BotError;

// ── CommandContext ────────────────────────────────────────────────────────────

/// Runtime context available to every [`BotCommand`](crate::command::BotCommand).
///
/// Carries the sender's identity, parsed arguments, and the channel reference
/// needed to send replies.
pub struct CommandContext {
    /// Full sender identifier, e.g. `"@alice:matrix.org"` or `"42"`.
    pub sender: String,
    /// Room or chat ID the message came from.
    pub room_id: String,
    /// Tokenized arguments after the command name.
    ///
    /// E.g. `"!deploy matrix prod"` → `args = ["matrix", "prod"]`.
    pub args: Vec<String>,
    /// The complete raw message body.
    pub raw: String,
    /// Permissions granted to this sender.
    pub permissions: PermissionSet,
    /// Channel to use for sending replies.
    pub channel: Arc<dyn Channel>,
}

impl CommandContext {
    /// Create a new context.
    pub fn new(
        sender: impl Into<String>,
        room_id: impl Into<String>,
        args: Vec<String>,
        raw: impl Into<String>,
        permissions: PermissionSet,
        channel: Arc<dyn Channel>,
    ) -> Self {
        Self {
            sender: sender.into(),
            room_id: room_id.into(),
            args,
            raw: raw.into(),
            permissions,
            channel,
        }
    }

    /// Send a plain-text reply to the room this command came from.
    pub async fn reply(&self, msg: impl Into<String>) -> Result<(), BotError> {
        self.channel
            .send(&self.room_id, ChannelMessage::text(msg))
            .await
            .map_err(|e| BotError::internal(e.to_string()))
    }

    /// Send a Markdown reply to the room this command came from.
    pub async fn reply_markdown(&self, msg: impl Into<String>) -> Result<(), BotError> {
        self.channel
            .send(&self.room_id, ChannelMessage::markdown(msg))
            .await
            .map_err(|e| BotError::internal(e.to_string()))
    }

    /// `true` when the sender holds `permission` (exact match or wildcard).
    pub fn has_permission(&self, permission: &Permission) -> bool {
        self.permissions.has_with_wildcard(permission)
    }
}
