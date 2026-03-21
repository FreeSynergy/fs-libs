//! Error type for the bridge executor.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum BridgeError {
    #[error("no bridge registered for role '{role}'")]
    NoBridgeForRole { role: String },

    #[error("method '{method}' not found in bridge '{bridge_id}'")]
    MethodNotFound { method: String, bridge_id: String },

    #[error("HTTP error calling {url}: {status}")]
    Http { url: String, status: String },

    #[error("request failed: {0}")]
    Reqwest(#[from] reqwest::Error),

    #[error("response JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("inventory error: {0}")]
    Inventory(String),
}
