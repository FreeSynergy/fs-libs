// fsn-channel/src/channel.rs — Core Channel trait.

use async_trait::async_trait;

use crate::error::ChannelError;
use crate::message::{ChannelMessage, IncomingMessage};

// ── Channel ───────────────────────────────────────────────────────────────────

/// Abstraction over a real-time messaging system (Matrix, Telegram, …).
///
/// # Implementing a new adapter
///
/// 1. Create a struct holding configuration and (optionally) a client handle.
/// 2. Implement `Channel` — all methods are async.
/// 3. Register the feature in `Cargo.toml` so optional deps compile only when needed.
///
/// # Example
///
/// ```rust,ignore
/// use fsn_channel::{Channel, ChannelMessage};
///
/// let ch = MatrixAdapter::new(MatrixConfig { … }).await?;
/// ch.connect().await?;
/// ch.send("#general:matrix.org", ChannelMessage::text("Hello!")).await?;
/// ```
#[async_trait]
pub trait Channel: Send + Sync {
    /// Short adapter name, e.g. `"matrix"`, `"telegram"`.
    fn adapter_name(&self) -> &str;

    /// Authenticate and open a persistent connection.
    ///
    /// Must be called before [`send`](Channel::send) or [`subscribe`](Channel::subscribe).
    async fn connect(&self) -> Result<(), ChannelError>;

    /// Send `message` to the given room or chat.
    ///
    /// `room_id` is adapter-specific:
    /// - Matrix: `"!roomid:homeserver.org"` or alias `"#general:matrix.org"`
    /// - Telegram: numeric chat ID as string, e.g. `"-100123456789"`
    async fn send(&self, room_id: &str, message: ChannelMessage) -> Result<(), ChannelError>;

    /// Subscribe to incoming messages.
    ///
    /// The `on_message` callback is called for every new message received.
    /// The adapter runs until an unrecoverable error occurs.
    ///
    /// The callback is boxed to keep this trait dyn-compatible.
    async fn subscribe(
        &self,
        on_message: Box<dyn Fn(IncomingMessage) + Send + Sync>,
    ) -> Result<(), ChannelError>;
}
