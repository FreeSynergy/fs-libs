// fsn-bot/src/router.rs — BotRouter bridges fsn-bus IncomingMessage events to commands.

use std::sync::Arc;

use async_trait::async_trait;
use fsn_auth::PermissionSet;
use fsn_bus::{BusError, Event, TopicHandler};
use fsn_channel::{Channel, IncomingMessage};
use tracing::{debug, warn};

use crate::context::CommandContext;
use crate::error::BotError;
use crate::registry::CommandRegistry;

// ── BotRouter ─────────────────────────────────────────────────────────────────

/// Bridges fsn-bus events to the bot [`CommandRegistry`].
///
/// Listens on `"channel.message.incoming"` events, parses the command prefix,
/// and dispatches to the appropriate [`BotCommand`](crate::command::BotCommand).
///
/// # Design
///
/// `BotRouter` implements [`TopicHandler`] so it can be registered directly
/// with the [`Router`](fsn_bus::Router). When an incoming message event arrives,
/// it:
/// 1. Deserialises the payload as [`IncomingMessage`].
/// 2. Checks whether the body starts with the configured prefix.
/// 3. Parses the command name and args.
/// 4. Looks up the command in the registry.
/// 5. Checks permissions.
/// 6. Calls `command.execute(ctx)`.
///
/// # Topic
///
/// Subscribes to `"channel.message.incoming"`.
pub struct BotRouter {
    /// Command prefix, e.g. `"!"`. Commands must start with this string.
    prefix: String,
    registry: CommandRegistry,
    channel: Arc<dyn Channel>,
    /// Permission resolver — maps sender IDs to [`PermissionSet`]s.
    permissions: Arc<dyn PermissionResolver>,
}

/// Resolves permissions for a sender.
///
/// Implement this to integrate with your auth system.
pub trait PermissionResolver: Send + Sync {
    /// Return the permissions granted to `sender_id`.
    fn resolve(&self, sender_id: &str) -> PermissionSet;
}

/// A [`PermissionResolver`] that grants all permissions to everyone.
///
/// **Only use this in development or testing.**
pub struct AllowAllPermissions;

impl PermissionResolver for AllowAllPermissions {
    fn resolve(&self, _sender_id: &str) -> PermissionSet {
        PermissionSet::new([
            fsn_auth::Permission::new("*"),
        ])
    }
}

/// A [`PermissionResolver`] that grants no permissions to anyone.
pub struct DenyAllPermissions;

impl PermissionResolver for DenyAllPermissions {
    fn resolve(&self, _sender_id: &str) -> PermissionSet {
        PermissionSet::default()
    }
}

impl BotRouter {
    /// Create a new router.
    ///
    /// - `prefix`      — command prefix, e.g. `"!"`
    /// - `registry`    — populated [`CommandRegistry`]
    /// - `channel`     — channel adapter used to send replies
    /// - `permissions` — permission resolver
    pub fn new(
        prefix: impl Into<String>,
        registry: CommandRegistry,
        channel: Arc<dyn Channel>,
        permissions: Arc<dyn PermissionResolver>,
    ) -> Self {
        Self { prefix: prefix.into(), registry, channel, permissions }
    }

    async fn handle_message(&self, msg: IncomingMessage) -> Result<(), BotError> {
        let body = msg.body.trim();

        if !body.starts_with(&self.prefix) {
            return Ok(());
        }

        let after_prefix = body[self.prefix.len()..].trim();
        let mut tokens = after_prefix.split_whitespace();
        let cmd_name = match tokens.next() {
            Some(n) => n,
            None    => return Ok(()),
        };
        let args: Vec<String> = tokens.map(String::from).collect();

        debug!(sender = %msg.sender, command = %cmd_name, "bot command received");

        let command = match self.registry.get(cmd_name) {
            Some(c) => c,
            None => {
                // Silently ignore unknown commands to avoid bot spam.
                return Ok(());
            }
        };

        let perms = self.permissions.resolve(&msg.sender);

        if let Some(required) = command.required_permission() {
            if !perms.has_with_wildcard(&required) {
                warn!(sender = %msg.sender, command = %cmd_name, "permission denied");
                return Err(BotError::permission_denied(cmd_name, required.as_str()));
            }
        }

        let ctx = CommandContext::new(
            msg.sender,
            msg.room_id,
            args,
            body,
            perms,
            self.channel.clone(),
        );

        command.execute(&ctx).await?;
        Ok(())
    }
}

#[async_trait]
impl TopicHandler for BotRouter {
    fn topic_pattern(&self) -> &str {
        "channel.message.incoming"
    }

    async fn handle(&self, event: &Event) -> Result<(), BusError> {
        let msg: IncomingMessage = event
            .parse_payload()
            .map_err(|e| BusError::handler(event.topic(), e.to_string()))?;

        self.handle_message(msg)
            .await
            .map_err(|e| BusError::handler(event.topic(), e.to_string()))
    }
}
