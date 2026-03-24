// fs-channel/src/telegram/mod.rs — Telegram adapter (feature: telegram).

pub mod config;

use async_trait::async_trait;
use teloxide::prelude::*;
use teloxide::types::ParseMode;
use tracing::{debug, info};

use crate::channel::Channel;
use crate::error::ChannelError;
use crate::message::{ChannelMessage, IncomingMessage};

pub use config::TelegramConfig;

// ── TelegramAdapter ───────────────────────────────────────────────────────────

/// Telegram Bot adapter built on [`teloxide`](https://github.com/teloxide/teloxide).
///
/// Requires feature `telegram`.
///
/// # Example
///
/// ```rust,ignore
/// use fs_channel::TelegramAdapter;
///
/// let adapter = TelegramAdapter::new(TelegramConfig {
///     bot_token: "123:token".into(),
///     allowed_chat_ids: vec![],
/// });
///
/// adapter.connect().await?;
/// adapter.send("-100123456789", ChannelMessage::text("Hello!")).await?;
/// ```
pub struct TelegramAdapter {
    config: TelegramConfig,
    bot: tokio::sync::OnceCell<Bot>,
}

impl TelegramAdapter {
    /// Create a new adapter. Call [`connect`](Channel::connect) before use.
    pub fn new(config: TelegramConfig) -> Self {
        Self {
            config,
            bot: tokio::sync::OnceCell::new(),
        }
    }

    fn bot(&self) -> Result<&Bot, ChannelError> {
        self.bot
            .get()
            .ok_or_else(|| ChannelError::connection("not connected — call connect() first"))
    }
}

#[async_trait]
impl Channel for TelegramAdapter {
    fn adapter_name(&self) -> &str {
        "telegram"
    }

    async fn connect(&self) -> Result<(), ChannelError> {
        let bot = Bot::new(&self.config.bot_token);
        // Verify the token works.
        bot.get_me()
            .await
            .map_err(|e| ChannelError::connection(e.to_string()))?;
        info!("telegram adapter connected");
        self.bot
            .set(bot)
            .map_err(|_| ChannelError::internal("bot already set"))?;
        Ok(())
    }

    async fn send(&self, room_id: &str, message: ChannelMessage) -> Result<(), ChannelError> {
        let bot = self.bot()?;
        let chat_id: i64 = room_id
            .parse()
            .map_err(|_| ChannelError::send(room_id, "invalid Telegram chat ID"))?;

        let chat = ChatId(chat_id);

        let req = bot.send_message(chat, message.rendered_body().as_ref());
        let req = if message.kind.is_rich() {
            req.parse_mode(ParseMode::MarkdownV2)
        } else {
            req
        };
        req.await
            .map_err(|e| ChannelError::send(room_id, e.to_string()))?;

        debug!(chat = %chat_id, "telegram message sent");
        Ok(())
    }

    async fn subscribe(
        &self,
        on_message: Box<dyn Fn(IncomingMessage) + Send + Sync>,
    ) -> Result<(), ChannelError> {
        let bot = self.bot()?.clone();
        let allowed = self.config.allowed_chat_ids.clone();

        let handler = Update::filter_message().endpoint(move |_bot: Bot, msg: Message| {
            let on_msg = &on_message;
            let allowed = &allowed;

            let chat_id = msg.chat.id.0;
            if !allowed.is_empty() && !allowed.contains(&chat_id) {
                return futures_util::future::ready(Ok(()));
            }

            let incoming = IncomingMessage {
                room_id: chat_id.to_string(),
                sender: msg
                    .from
                    .as_ref()
                    .map(|u| u.username.clone().unwrap_or_else(|| u.id.to_string()))
                    .unwrap_or_default(),
                body: msg.text().unwrap_or("").to_string(),
                timestamp: chrono::Utc::now(),
            };
            on_msg(incoming);
            futures_util::future::ready(Ok(()))
        });

        Dispatcher::builder(bot, handler).build().dispatch().await;

        Ok(())
    }
}
