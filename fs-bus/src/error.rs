// fs-bus/src/error.rs — Error types for the event bus.

use thiserror::Error;

/// Errors produced by the fs-bus event routing and delivery system.
#[derive(Error, Debug, Clone)]
pub enum BusError {
    /// JSON serialization or deserialization failed.
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// A handler returned an error while processing an event.
    #[error("Handler error on topic '{topic}': {message}")]
    Handler { topic: String, message: String },

    /// Template rendering failed (requires `tera-transform` feature).
    #[error("Transform error: {0}")]
    Transform(String),

    /// An event could not be delivered after all retry attempts.
    #[error("Delivery failed after {attempts} attempt(s): {last_error}")]
    Retry { attempts: u32, last_error: String },

    /// A catch-all for unexpected internal failures.
    #[error("Internal bus error: {0}")]
    Internal(String),
}

impl BusError {
    /// Convenience constructor for serialization errors.
    pub fn serialization(msg: impl Into<String>) -> Self {
        Self::Serialization(msg.into())
    }

    /// Convenience constructor for handler errors.
    pub fn handler(topic: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Handler { topic: topic.into(), message: message.into() }
    }

    /// Convenience constructor for transform errors.
    pub fn transform(msg: impl Into<String>) -> Self {
        Self::Transform(msg.into())
    }

    /// Convenience constructor for internal errors.
    pub fn internal(msg: impl Into<String>) -> Self {
        Self::Internal(msg.into())
    }
}
