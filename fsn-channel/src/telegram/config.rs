// fsn-channel/src/telegram/config.rs — Telegram adapter configuration.

use serde::{Deserialize, Serialize};

/// Configuration for the Telegram adapter.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelegramConfig {
    /// Bot API token from @BotFather, e.g. `"123456:ABC-..."`.
    #[serde(skip_serializing)]
    pub bot_token: String,
    /// Optional: only process updates from these chat IDs (empty = allow all).
    #[serde(default)]
    pub allowed_chat_ids: Vec<i64>,
}
