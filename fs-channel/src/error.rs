// fs-channel/src/error.rs — Error types for channel adapters.

use thiserror::Error;

/// Errors produced by channel adapters.
#[derive(Error, Debug)]
pub enum ChannelError {
    /// Connection or authentication failure.
    #[error("Connection error: {0}")]
    Connection(String),

    /// Message send failed.
    #[error("Send error to '{room}': {message}")]
    Send { room: String, message: String },

    /// Incoming message stream error.
    #[error("Receive error: {0}")]
    Receive(String),

    /// Configuration is invalid.
    #[error("Configuration error: {0}")]
    Config(String),

    /// Catch-all for unexpected failures.
    #[error("Internal channel error: {0}")]
    Internal(String),
}

impl ChannelError {
    /// Convenience constructor for connection errors.
    pub fn connection(msg: impl Into<String>) -> Self {
        Self::Connection(msg.into())
    }

    /// Convenience constructor for send errors.
    pub fn send(room: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Send { room: room.into(), message: message.into() }
    }

    /// Convenience constructor for receive errors.
    pub fn receive(msg: impl Into<String>) -> Self {
        Self::Receive(msg.into())
    }

    /// Convenience constructor for config errors.
    pub fn config(msg: impl Into<String>) -> Self {
        Self::Config(msg.into())
    }

    /// Convenience constructor for internal errors.
    pub fn internal(msg: impl Into<String>) -> Self {
        Self::Internal(msg.into())
    }
}
