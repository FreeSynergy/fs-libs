//! `MessengerAdapterResource` — a store package that implements the Channel trait for one messenger platform.

use super::bot::TokenDef;
use super::meta::ResourceMeta;
use super::widget::RoleRequirement;
use serde::{Deserialize, Serialize};

// ── MessengerKind ─────────────────────────────────────────────────────────────

/// All supported messenger platforms.
///
/// Each variant corresponds to one `Channel` adapter implementation in `fsn-channel`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MessengerKind {
    /// Telegram bot API / MTProto.
    Telegram,
    /// Matrix Client-Server API.
    Matrix,
    /// Discord bot API.
    Discord,
    /// Rocket.Chat REST API.
    RocketChat,
    /// Mattermost REST API.
    Mattermost,
    /// XMPP / Jabber (MUC).
    Xmpp,
    /// Zulip REST API.
    Zulip,
    /// Revolt REST API.
    Revolt,
    /// Nextcloud Talk REST API.
    NextcloudTalk,
    /// IRC (via bridge or REST).
    Irc,
    /// Slack Web API.
    Slack,
    /// Microsoft Teams via Graph API.
    Teams,
    /// Viber bot API.
    Viber,
    /// LINE Messaging API.
    Line,
    /// WhatsApp Cloud API (Meta).
    WhatsApp,
    /// Signal via signal-cli REST API.
    Signal,
    /// Threema Gateway API.
    Threema,
    /// Wire bot API.
    Wire,
    /// Discourse API.
    Discourse,
    /// Lemmy API.
    Lemmy,
    /// Mastodon API.
    Mastodon,
}

impl MessengerKind {
    /// Human-readable display name.
    pub fn label(self) -> &'static str {
        match self {
            MessengerKind::Telegram      => "Telegram",
            MessengerKind::Matrix        => "Matrix",
            MessengerKind::Discord       => "Discord",
            MessengerKind::RocketChat    => "Rocket.Chat",
            MessengerKind::Mattermost    => "Mattermost",
            MessengerKind::Xmpp          => "XMPP",
            MessengerKind::Zulip         => "Zulip",
            MessengerKind::Revolt        => "Revolt",
            MessengerKind::NextcloudTalk => "Nextcloud Talk",
            MessengerKind::Irc           => "IRC",
            MessengerKind::Slack         => "Slack",
            MessengerKind::Teams         => "Microsoft Teams",
            MessengerKind::Viber         => "Viber",
            MessengerKind::Line          => "LINE",
            MessengerKind::WhatsApp      => "WhatsApp",
            MessengerKind::Signal        => "Signal",
            MessengerKind::Threema       => "Threema",
            MessengerKind::Wire          => "Wire",
            MessengerKind::Discourse     => "Discourse",
            MessengerKind::Lemmy         => "Lemmy",
            MessengerKind::Mastodon      => "Mastodon",
        }
    }

    /// Whether this adapter requires the platform to be self-hosted.
    pub fn is_self_hosted_only(self) -> bool {
        matches!(
            self,
            MessengerKind::RocketChat
                | MessengerKind::Mattermost
                | MessengerKind::Xmpp
                | MessengerKind::Zulip
                | MessengerKind::Revolt
                | MessengerKind::NextcloudTalk
                | MessengerKind::Irc
                | MessengerKind::Discourse
                | MessengerKind::Lemmy
                | MessengerKind::Matrix
        )
    }

    /// Rust feature flag name required to compile this adapter.
    ///
    /// Returns `None` for REST-only adapters that compile without extra dependencies.
    pub fn required_feature(self) -> Option<&'static str> {
        match self {
            MessengerKind::Telegram      => Some("telegram"),
            MessengerKind::Matrix        => Some("matrix"),
            MessengerKind::Discord       => Some("discord"),
            MessengerKind::Xmpp          => Some("xmpp"),
            MessengerKind::Irc           => Some("irc"),
            MessengerKind::Slack         => Some("slack"),
            MessengerKind::Signal        => Some("signal"),
            MessengerKind::Mastodon      => Some("mastodon"),
            _ => None, // REST adapters compile without extra features
        }
    }
}

// ── AdapterAuthMethod ─────────────────────────────────────────────────────────

/// How the adapter authenticates with the messenger platform.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AdapterAuthMethod {
    /// A single long-lived bot token (Telegram, Discord, Slack, …).
    BotToken,
    /// OAuth 2.0 client credentials flow.
    OAuth2,
    /// A static API key header.
    ApiKey,
    /// Username + password.
    UserPassword,
    /// MTProto user-session (Telegram user-bot mode).
    MtProto,
    /// Gateway account credentials (Threema, WhatsApp Business, …).
    GatewayCredentials,
}

impl AdapterAuthMethod {
    /// Human-readable label.
    pub fn label(self) -> &'static str {
        match self {
            AdapterAuthMethod::BotToken           => "Bot Token",
            AdapterAuthMethod::OAuth2             => "OAuth 2.0",
            AdapterAuthMethod::ApiKey             => "API Key",
            AdapterAuthMethod::UserPassword       => "Username + Password",
            AdapterAuthMethod::MtProto            => "MTProto Session",
            AdapterAuthMethod::GatewayCredentials => "Gateway Credentials",
        }
    }
}

// ── ChannelFeature ────────────────────────────────────────────────────────────

/// A capability of the `Channel` trait that an adapter may or may not support.
///
/// Adapters declare which features they implement so that bot logic can
/// gracefully degrade when a platform does not support an operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChannelFeature {
    /// `Channel::create_room`
    CreateRoom,
    /// `Channel::invite`
    Invite,
    /// `Channel::kick`
    Kick,
    /// `Channel::send`
    Send,
    /// `Channel::delete_room`
    DeleteRoom,
    /// `Channel::get_members`
    GetMembers,
}

impl ChannelFeature {
    /// Human-readable label.
    pub fn label(self) -> &'static str {
        match self {
            ChannelFeature::CreateRoom => "Create Room",
            ChannelFeature::Invite     => "Invite",
            ChannelFeature::Kick       => "Kick",
            ChannelFeature::Send       => "Send Message",
            ChannelFeature::DeleteRoom => "Delete Room",
            ChannelFeature::GetMembers => "Get Members",
        }
    }
}

// ── MessengerAdapterResource ──────────────────────────────────────────────────

/// A store package that provides a `Channel` adapter for one messenger platform.
///
/// Installed via the Store as `resource_type = "messenger_adapter"`.
/// The Inventory's `services_with_role("chat")` returns active adapter instances.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessengerAdapterResource {
    /// Common metadata present on every resource.
    pub meta: ResourceMeta,
    /// Which messenger platform this adapter implements.
    pub kind: MessengerKind,
    /// How this adapter authenticates with the platform.
    pub auth_method: AdapterAuthMethod,
    /// Credentials / tokens this adapter needs at runtime.
    pub tokens_required: Vec<TokenDef>,
    /// Which `Channel` trait methods this adapter implements.
    pub supported_features: Vec<ChannelFeature>,
    /// Base URL for self-hosted platforms (e.g. `"https://chat.mycompany.com"`).
    /// `None` for cloud-hosted platforms where the URL is fixed.
    pub api_base_url: Option<String>,
    /// FSN roles that must be available for this adapter to be active.
    pub required_roles: Vec<RoleRequirement>,
}

impl MessengerAdapterResource {
    /// Returns `true` if this adapter implements the given feature.
    pub fn supports(&self, feature: ChannelFeature) -> bool {
        self.supported_features.contains(&feature)
    }
}
