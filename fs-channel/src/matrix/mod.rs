// fs-channel/src/matrix/mod.rs — Matrix adapter (feature: matrix).

pub mod config;

use async_trait::async_trait;
use matrix_sdk::{
    config::SyncSettings,
    room::Room,
    ruma::events::room::message::{
        MessageType, OriginalSyncRoomMessageEvent, RoomMessageEventContent,
    },
    Client,
};
use tracing::{debug, info};

use crate::channel::Channel;
use crate::error::ChannelError;
use crate::message::{ChannelMessage, IncomingMessage};

pub use config::MatrixConfig;

// ── MatrixAdapter ─────────────────────────────────────────────────────────────

/// Matrix messaging adapter built on [`matrix-sdk`](https://github.com/matrix-org/matrix-rust-sdk).
///
/// Requires feature `matrix`.
///
/// # Example
///
/// ```rust,ignore
/// use fs_channel::MatrixAdapter;
///
/// let adapter = MatrixAdapter::new(MatrixConfig {
///     homeserver_url: "https://matrix.example.org".into(),
///     user_id: "@fs-bot:example.org".into(),
///     password: Some("secret".into()),
///     access_token: None,
///     store_path: "./session".into(),
/// });
///
/// adapter.connect().await?;
/// adapter.send("!room:example.org", ChannelMessage::text("Hello!")).await?;
/// ```
pub struct MatrixAdapter {
    config: MatrixConfig,
    client: tokio::sync::OnceCell<Client>,
}

impl MatrixAdapter {
    /// Create a new adapter from the given config. Call [`connect`](Channel::connect) before use.
    pub fn new(config: MatrixConfig) -> Self {
        Self {
            config,
            client: tokio::sync::OnceCell::new(),
        }
    }

    fn client(&self) -> Result<&Client, ChannelError> {
        self.client
            .get()
            .ok_or_else(|| ChannelError::connection("not connected — call connect() first"))
    }

    fn build_content(msg: &ChannelMessage) -> RoomMessageEventContent {
        if msg.kind.is_rich() {
            RoomMessageEventContent::text_markdown(msg.rendered_body().as_ref())
        } else {
            RoomMessageEventContent::text_plain(&msg.body)
        }
    }
}

#[async_trait]
impl Channel for MatrixAdapter {
    fn adapter_name(&self) -> &str {
        "matrix"
    }

    async fn connect(&self) -> Result<(), ChannelError> {
        let client = Client::builder()
            .homeserver_url(&self.config.homeserver_url)
            .build()
            .await
            .map_err(|e| ChannelError::connection(e.to_string()))?;

        if let Some(token) = &self.config.access_token {
            // Restore existing session
            client
                .restore_session(matrix_sdk::AuthSession::Matrix(
                    matrix_sdk::authentication::matrix::MatrixSession {
                        tokens: matrix_sdk::authentication::SessionTokens {
                            access_token: token.clone(),
                            refresh_token: None,
                        },
                        meta: matrix_sdk::SessionMeta {
                            user_id: self.config.user_id.parse().map_err(
                                |e: matrix_sdk::IdParseError| ChannelError::config(e.to_string()),
                            )?,
                            device_id: "FSN".into(),
                        },
                    },
                ))
                .await
                .map_err(|e| ChannelError::connection(e.to_string()))?;
        } else if let Some(password) = &self.config.password {
            client
                .matrix_auth()
                .login_username(&self.config.user_id, password)
                .initial_device_display_name("FreeSynergy Bot")
                .send()
                .await
                .map_err(|e| ChannelError::connection(e.to_string()))?;
        } else {
            return Err(ChannelError::config(
                "either access_token or password must be set in MatrixConfig",
            ));
        }

        info!(user = %self.config.user_id, "matrix adapter connected");
        self.client
            .set(client)
            .map_err(|_| ChannelError::internal("client already set"))?;
        Ok(())
    }

    async fn send(&self, room_id: &str, message: ChannelMessage) -> Result<(), ChannelError> {
        let client = self.client()?;
        let room_id: matrix_sdk::ruma::OwnedRoomId = room_id
            .parse()
            .map_err(|e: matrix_sdk::IdParseError| ChannelError::send(room_id, e.to_string()))?;

        let room = client
            .get_room(&room_id)
            .ok_or_else(|| ChannelError::send(room_id.as_str(), "room not joined"))?;

        let content = Self::build_content(&message);
        room.send(content)
            .await
            .map_err(|e| ChannelError::send(room_id.as_str(), e.to_string()))?;

        debug!(room = %room_id, "matrix message sent");
        Ok(())
    }

    async fn subscribe(
        &self,
        on_message: Box<dyn Fn(IncomingMessage) + Send + Sync>,
    ) -> Result<(), ChannelError> {
        let client = self.client()?.clone();

        client.add_event_handler(move |ev: OriginalSyncRoomMessageEvent, room: Room| {
            let body = match &ev.content.msgtype {
                MessageType::Text(t) => t.body.clone(),
                MessageType::Notice(n) => n.body.clone(),
                _ => return futures_util::future::ready(()),
            };

            let incoming = IncomingMessage {
                room_id: room.room_id().to_string(),
                sender: ev.sender.to_string(),
                body,
                timestamp: chrono::Utc::now(),
            };
            on_message(incoming);
            futures_util::future::ready(())
        });

        client
            .sync(SyncSettings::default())
            .await
            .map_err(|e| ChannelError::receive(e.to_string()))?;

        Ok(())
    }
}
