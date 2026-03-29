//! `BotResource` — a messaging bot that reacts to Bus events.

use super::meta::{ResourceMeta, Role};
use super::widget::RoleRequirement;
use serde::{Deserialize, Serialize};

// ── ChannelType ───────────────────────────────────────────────────────────────

/// Messaging platform / channel this bot can connect to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChannelType {
    Telegram,
    Matrix,
    Discord,
    Slack,
    Signal,
    Webhook,
}

impl ChannelType {
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            ChannelType::Telegram => "Telegram",
            ChannelType::Matrix => "Matrix",
            ChannelType::Discord => "Discord",
            ChannelType::Slack => "Slack",
            ChannelType::Signal => "Signal",
            ChannelType::Webhook => "Webhook",
        }
    }
}

// ── BotCommand ────────────────────────────────────────────────────────────────

/// A slash-command the bot understands, e.g. `/broadcast`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BotCommand {
    /// Command trigger without the `/` prefix, e.g. `"broadcast"`.
    pub name: String,
    /// Short description shown in the channel's command picker.
    pub description: String,
    /// Whether admin permission is required to run this command.
    pub requires_admin: bool,
}

// ── BusTrigger ────────────────────────────────────────────────────────────────

/// A Bus event topic that causes this bot to react.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BusTrigger {
    /// Bus topic pattern, e.g. `"user.joined"` or `"node.alert.*"`.
    pub topic: String,
    /// Human-readable description of what action the bot takes.
    pub action_description: String,
}

// ── TokenDef ─────────────────────────────────────────────────────────────────

/// An API token or credential the bot needs at runtime.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenDef {
    /// Variable name used in the bot's config, e.g. `"TELEGRAM_BOT_TOKEN"`.
    pub name: String,
    /// Where to obtain the token, e.g. `"@BotFather on Telegram"`.
    pub source_hint: String,
    /// When `false`, the bot refuses to start without this token.
    pub optional: bool,
}

// ── BotResource ──────────────────────────────────────────────────────────────

/// A bot that bridges messaging channels with `FreeSynergy` Bus events.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BotResource {
    /// Shared metadata present on every resource.
    pub meta: ResourceMeta,
    /// Messaging channels this bot can connect to.
    pub channels: Vec<ChannelType>,
    /// Slash-commands the bot understands.
    pub commands: Vec<BotCommand>,
    /// Role groups that must be satisfied for this bot to function.
    pub required_roles: Vec<RoleRequirement>,
    /// Bus event topics this bot reacts to.
    pub triggers: Vec<BusTrigger>,
    /// Roles this bot provides when running.
    pub roles_provided: Vec<Role>,
    /// API tokens / credentials required at runtime.
    pub tokens_required: Vec<TokenDef>,
}
