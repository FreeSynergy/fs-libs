//! Messaging channel abstraction for FreeSynergy.
//!
//! Provides a unified [`Channel`] trait for sending and receiving messages
//! across different messaging platforms.
//!
//! # Features
//!
//! | feature    | adds               | extra dep       |
//! |------------|--------------------|-----------------|
//! | `matrix`   | [`MatrixAdapter`]  | `matrix-sdk`    |
//! | `telegram` | [`TelegramAdapter`]| `teloxide`      |
//!
//! # Quick start
//!
//! ```rust,ignore
//! use fsn_channel::{Channel, ChannelMessage, MatrixAdapter, MatrixConfig};
//!
//! let adapter = MatrixAdapter::new(MatrixConfig { … });
//! adapter.connect().await?;
//! adapter.send("!room:matrix.org", ChannelMessage::text("Hello!")).await?;
//! ```

pub mod channel;
pub mod error;
pub mod message;

#[cfg(feature = "matrix")]
pub mod matrix;

#[cfg(feature = "telegram")]
pub mod telegram;

// ── Flat re-exports ───────────────────────────────────────────────────────────

pub use channel::Channel;
pub use error::ChannelError;
pub use message::{Attachment, ChannelMessage, IncomingMessage, MessageKind};

#[cfg(feature = "matrix")]
pub use matrix::{MatrixAdapter, MatrixConfig};

#[cfg(feature = "telegram")]
pub use telegram::{TelegramAdapter, TelegramConfig};
