// fs-llm/src/error.rs — Error types for LLM providers.

use thiserror::Error;

/// Errors produced by LLM provider calls and task functions.
#[derive(Error, Debug)]
pub enum LlmError {
    /// HTTP transport failure.
    #[error("Network error: {0}")]
    Network(String),

    /// The provider returned a non-success status.
    #[error("Provider error ({status}): {message}")]
    Provider { status: u16, message: String },

    /// JSON serialization or deserialization failed.
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// The response did not contain expected content.
    #[error("Empty response from provider")]
    EmptyResponse,

    /// A task function could not parse the LLM output.
    #[error("Parse error in task output: {0}")]
    TaskParse(String),

    /// Catch-all for unexpected internal failures.
    #[error("Internal error: {0}")]
    Internal(String),
}

impl LlmError {
    /// Convenience constructor for network errors.
    pub fn network(msg: impl Into<String>) -> Self {
        Self::Network(msg.into())
    }

    /// Convenience constructor for provider errors.
    pub fn provider(status: u16, message: impl Into<String>) -> Self {
        Self::Provider { status, message: message.into() }
    }

    /// Convenience constructor for serialization errors.
    pub fn serialization(msg: impl Into<String>) -> Self {
        Self::Serialization(msg.into())
    }

    /// Convenience constructor for task parse errors.
    pub fn task_parse(msg: impl Into<String>) -> Self {
        Self::TaskParse(msg.into())
    }

    /// Convenience constructor for internal errors.
    pub fn internal(msg: impl Into<String>) -> Self {
        Self::Internal(msg.into())
    }
}
