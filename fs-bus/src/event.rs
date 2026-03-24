// fs-bus/src/event.rs — Core event types.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::BusError;

// ── EventId ───────────────────────────────────────────────────────────────────

/// Unique identifier for a single bus event.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EventId(Uuid);

impl EventId {
    /// Generate a new random event ID.
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Return the underlying UUID.
    pub fn as_uuid(&self) -> &Uuid {
        &self.0
    }
}

impl Default for EventId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for EventId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

// ── EventMeta ─────────────────────────────────────────────────────────────────

/// Metadata carried by every event — id, topic, timestamp, source.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventMeta {
    /// Unique event identifier.
    pub id: EventId,
    /// Dot-separated topic path, e.g. `"deploy.started"`, `"health.check"`.
    pub topic: String,
    /// UTC timestamp when the event was created.
    pub timestamp: DateTime<Utc>,
    /// Component or service that produced this event, e.g. `"fs-node"`.
    pub source: String,
}

impl EventMeta {
    /// Create new metadata for `topic` from `source`.
    pub fn new(topic: impl Into<String>, source: impl Into<String>) -> Self {
        Self {
            id: EventId::new(),
            topic: topic.into(),
            timestamp: Utc::now(),
            source: source.into(),
        }
    }
}

// ── Event ─────────────────────────────────────────────────────────────────────

/// A bus event — metadata plus a JSON payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    /// Routing and identification metadata.
    pub meta: EventMeta,
    /// Arbitrary JSON payload.
    pub payload: serde_json::Value,
}

impl Event {
    /// Build a new event for `topic` emitted by `source` with `payload`.
    ///
    /// # Errors
    ///
    /// Returns [`BusError::Serialization`] if `payload` cannot be serialized.
    pub fn new(
        topic: impl Into<String>,
        source: impl Into<String>,
        payload: impl Serialize,
    ) -> Result<Self, BusError> {
        let payload =
            serde_json::to_value(payload).map_err(|e| BusError::serialization(e.to_string()))?;
        Ok(Self {
            meta: EventMeta::new(topic, source),
            payload,
        })
    }

    /// Return the topic string.
    pub fn topic(&self) -> &str {
        &self.meta.topic
    }

    /// Deserialize the payload as `T`.
    ///
    /// # Errors
    ///
    /// Returns [`BusError::Serialization`] if the payload does not match `T`.
    pub fn parse_payload<T: serde::de::DeserializeOwned>(&self) -> Result<T, BusError> {
        serde_json::from_value(self.payload.clone())
            .map_err(|e| BusError::serialization(e.to_string()))
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    struct Ping {
        message: String,
    }

    #[test]
    fn event_round_trip() {
        let ev = Event::new(
            "ping",
            "test",
            Ping {
                message: "hello".into(),
            },
        )
        .unwrap();
        assert_eq!(ev.topic(), "ping");
        let p: Ping = ev.parse_payload().unwrap();
        assert_eq!(p.message, "hello");
    }

    #[test]
    fn event_id_is_unique() {
        let a = EventId::new();
        let b = EventId::new();
        assert_ne!(a, b);
    }
}
