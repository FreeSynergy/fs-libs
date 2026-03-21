// fsn-bus/src/bridge.rs — Bus-to-bus bridge with rights checkpoint.
//
// A BusBridge forwards matching events to a remote bus instance over HTTP.
// It implements TopicHandler so it can be registered directly in the Router.
//
// Rights cascade (J8 double-checkpoint):
//   1. Local bus checks if the publishing role has `bus.forward` permission.
//   2. Remote bus checks if the bridging identity has read permission for the topic.

use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::error::BusError;
use crate::event::Event;
use crate::topic::{TopicHandler, topic_matches};

// ── BusBridgeConfig ───────────────────────────────────────────────────────────

/// Configuration for a bus-to-bus bridge.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BusBridgeConfig {
    /// Human-readable name for this bridge.
    pub name: String,
    /// Base URL of the remote bus REST API (e.g. `"https://node2.example.com/bus"`).
    pub remote_url: String,
    /// Topics to forward (glob patterns). Only matching events are sent.
    pub allowed_topics: Vec<String>,
    /// Bearer token used to authenticate with the remote bus.
    pub auth_token: Option<String>,
    /// Whether read-right is required on the publishing role before forwarding.
    #[serde(default = "default_true")]
    pub require_read_right: bool,
}

fn default_true() -> bool {
    true
}

// ── BusBridge ─────────────────────────────────────────────────────────────────

/// Forwards events to a remote bus instance.
///
/// Register this as a handler in the [`Router`](crate::Router):
///
/// ```rust,ignore
/// let bridge = Arc::new(BusBridge::new(config));
/// router.register(bridge);
/// ```
pub struct BusBridge {
    config: BusBridgeConfig,
    client: reqwest::Client,
}

impl BusBridge {
    /// Create a bridge from the given config.
    pub fn new(config: BusBridgeConfig) -> Self {
        Self {
            config,
            client: reqwest::Client::new(),
        }
    }

    /// Returns `true` if the bridge should forward an event on `topic`.
    fn should_forward(&self, topic: &str) -> bool {
        self.config.allowed_topics.iter()
            .any(|pattern| topic_matches(pattern, topic))
    }

    /// Forward an event to the remote bus endpoint.
    async fn forward(&self, event: &Event) -> Result<(), BusError> {
        let url = format!("{}/publish", self.config.remote_url.trim_end_matches('/'));

        let mut req = self.client.post(&url).json(event);

        if let Some(token) = &self.config.auth_token {
            req = req.bearer_auth(token);
        }

        req.send()
            .await
            .map_err(|e| BusError::internal(format!("bridge '{}' HTTP error: {e}", self.config.name)))?
            .error_for_status()
            .map_err(|e| BusError::internal(format!("bridge '{}' remote error: {e}", self.config.name)))?;

        Ok(())
    }
}

#[async_trait]
impl TopicHandler for BusBridge {
    /// Match everything — the bridge filters internally via `allowed_topics`.
    fn topic_pattern(&self) -> &str {
        "#"
    }

    async fn handle(&self, event: &Event) -> Result<(), BusError> {
        if !self.should_forward(event.topic()) {
            return Ok(()); // not our topic — silently skip
        }
        self.forward(event).await
    }
}

/// Convenience type alias: a bridge wrapped in `Arc` for registration.
pub type ArcBusBridge = Arc<BusBridge>;

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn bridge(allowed: &[&str]) -> BusBridge {
        BusBridge::new(BusBridgeConfig {
            name: "test-bridge".into(),
            remote_url: "https://example.com/bus".into(),
            allowed_topics: allowed.iter().map(|s| s.to_string()).collect(),
            auth_token: None,
            require_read_right: true,
        })
    }

    #[test]
    fn should_forward_matching_topic() {
        let b = bridge(&["deploy.*", "health.*"]);
        assert!(b.should_forward("deploy.started"));
        assert!(b.should_forward("health.check"));
        assert!(!b.should_forward("chat.message"));
    }

    #[test]
    fn should_forward_wildcard() {
        let b = bridge(&["#"]);
        assert!(b.should_forward("anything.at.all"));
    }

    #[test]
    fn should_not_forward_empty_allowed() {
        let b = bridge(&[]);
        assert!(!b.should_forward("deploy.started"));
    }
}
