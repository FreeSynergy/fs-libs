// fs-bus/src/standing_order.rs — Standing orders: persistent trigger rules.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use crate::error::BusError;
use crate::event::Event;
use crate::topic::topic_matches;

// ── StandingOrder ─────────────────────────────────────────────────────────────

/// A persistent trigger that fires an event whenever a matching condition is met.
///
/// Standing orders are stored in the DB and survive restarts. The engine checks
/// them whenever:
/// - A new service of `trigger_role` is installed/appears on the bus.
/// - An event on `topic` arrives that matches the order's filter (for
///   recurring triggers).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StandingOrder {
    /// Unique identifier.
    pub id: Uuid,
    /// Human-readable name for this standing order.
    pub name: String,
    /// The role whose appearance triggers this order (e.g. `"chat"`, `"iam"`).
    /// When a service advertising this role connects, the order fires.
    pub trigger_role: String,
    /// Topic to publish when the order fires.
    pub topic: String,
    /// JSON payload for the generated event.
    pub payload: Value,
    /// Whether this standing order is active.
    pub enabled: bool,
}

impl StandingOrder {
    /// Create a new enabled standing order.
    pub fn new(
        name: impl Into<String>,
        trigger_role: impl Into<String>,
        topic: impl Into<String>,
        payload: Value,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            trigger_role: trigger_role.into(),
            topic: topic.into(),
            payload,
            enabled: true,
        }
    }

    /// Create an event from this standing order, attributed to `source`.
    ///
    /// # Errors
    /// Returns [`BusError`] if payload serialization fails.
    pub fn to_event(&self, source: &str) -> Result<Event, BusError> {
        Ok(Event {
            meta: crate::event::EventMeta::new(self.topic.clone(), source),
            payload: self.payload.clone(),
        })
    }
}

// ── StandingOrdersEngine ──────────────────────────────────────────────────────

/// Manages the set of active standing orders and evaluates trigger conditions.
///
/// The engine is in-memory. Load from DB at startup via [`add`](Self::add).
#[derive(Debug, Default)]
pub struct StandingOrdersEngine {
    orders: Vec<StandingOrder>,
}

impl StandingOrdersEngine {
    /// Create an empty engine.
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a standing order.
    pub fn add(&mut self, order: StandingOrder) {
        self.orders.push(order);
    }

    /// Remove a standing order by ID. Returns `true` if it existed.
    pub fn remove(&mut self, id: Uuid) -> bool {
        let before = self.orders.len();
        self.orders.retain(|o| o.id != id);
        self.orders.len() < before
    }

    /// Enable or disable a standing order by ID. Returns `true` if found.
    pub fn set_enabled(&mut self, id: Uuid, enabled: bool) -> bool {
        if let Some(o) = self.orders.iter_mut().find(|o| o.id == id) {
            o.enabled = enabled;
            true
        } else {
            false
        }
    }

    /// Generate events for all standing orders triggered by a new `role` appearing.
    ///
    /// Call this when a service advertising `role` connects or is installed.
    /// Returns one [`Event`] per enabled matching order.
    pub fn trigger_for_role(&self, role: &str, source: &str) -> Vec<Result<Event, BusError>> {
        self.orders
            .iter()
            .filter(|o| o.enabled && o.trigger_role == role)
            .map(|o| o.to_event(source))
            .collect()
    }

    /// Generate events for all standing orders whose topic matches `topic`.
    ///
    /// Used for recurring triggers: when an event on `topic` arrives the engine
    /// may fire additional orders that react to it.
    pub fn trigger_for_topic(&self, topic: &str, source: &str) -> Vec<Result<Event, BusError>> {
        self.orders
            .iter()
            .filter(|o| o.enabled && topic_matches(&o.topic, topic))
            .map(|o| o.to_event(source))
            .collect()
    }

    /// Number of registered standing orders.
    pub fn len(&self) -> usize {
        self.orders.len()
    }

    /// Returns `true` if there are no registered standing orders.
    pub fn is_empty(&self) -> bool {
        self.orders.is_empty()
    }

    /// Iterate over all standing orders.
    pub fn iter(&self) -> impl Iterator<Item = &StandingOrder> {
        self.orders.iter()
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn order(name: &str, role: &str, topic: &str) -> StandingOrder {
        StandingOrder::new(name, role, topic, json!({"hello": "world"}))
    }

    #[test]
    fn trigger_for_role_matching() {
        let mut eng = StandingOrdersEngine::new();
        eng.add(order("greet-chat", "chat", "system.hello"));
        eng.add(order("greet-iam", "iam", "system.hello"));

        let events = eng.trigger_for_role("chat", "fs-bus");
        assert_eq!(events.len(), 1);
        let ev = events[0].as_ref().unwrap();
        assert_eq!(ev.topic(), "system.hello");
    }

    #[test]
    fn trigger_for_role_disabled() {
        let mut eng = StandingOrdersEngine::new();
        let mut o = order("greet", "chat", "system.hello");
        o.enabled = false;
        eng.add(o);

        assert!(eng.trigger_for_role("chat", "fs-bus").is_empty());
    }

    #[test]
    fn remove_order() {
        let mut eng = StandingOrdersEngine::new();
        let o = order("greet", "chat", "system.hello");
        let id = o.id;
        eng.add(o);
        assert!(eng.remove(id));
        assert!(eng.is_empty());
    }

    #[test]
    fn set_enabled_toggle() {
        let mut eng = StandingOrdersEngine::new();
        let o = order("greet", "chat", "system.hello");
        let id = o.id;
        eng.add(o);

        eng.set_enabled(id, false);
        assert!(eng.trigger_for_role("chat", "test").is_empty());

        eng.set_enabled(id, true);
        assert_eq!(eng.trigger_for_role("chat", "test").len(), 1);
    }
}
