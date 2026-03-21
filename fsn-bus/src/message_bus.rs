// fsn-bus/src/message_bus.rs — Main MessageBus orchestrator (J1-J7).
//
// Ties together:
//   - Router (topic dispatch)
//   - SubscriptionManager (role → topic filter registry)
//   - StandingOrdersEngine (persistent trigger rules)
//   - RoutingConfig (TOML-based delivery/storage rules)
//
// Storage (event_log, subscriptions table) is handled externally — consumers
// should persist events via the `on_event` callback or a BusStore trait impl.

use std::sync::Arc;

use crate::config::RoutingConfig;
use crate::error::BusError;
use crate::event::Event;
use crate::message::{BusMessage, DeliveryType, StorageType};
use crate::router::Router;
use crate::standing_order::{StandingOrder, StandingOrdersEngine};
use crate::subscription::{Subscription, SubscriptionManager};
use crate::topic::TopicHandler;

// ── PublishedEvent ────────────────────────────────────────────────────────────

/// Metadata about an event after it has been published, returned by [`MessageBus::publish`].
#[derive(Debug, Clone)]
pub struct PublishedEvent {
    /// The message that was published.
    pub message: BusMessage,
    /// The resolved delivery type (from config or explicit).
    pub delivery: DeliveryType,
    /// The resolved storage type (from config or explicit).
    pub storage: StorageType,
    /// Roles that received the event (matched subscriptions).
    pub delivered_to: Vec<String>,
    /// Handler results (one per matching topic handler in the router).
    pub handler_results: Vec<Result<(), BusError>>,
}

// ── MessageBus ────────────────────────────────────────────────────────────────

/// The main bus: publish events, manage subscriptions, fire standing orders.
///
/// # Usage
///
/// ```rust,ignore
/// let mut bus = MessageBus::default();
/// bus.subscribe(Subscription::new("chat", "chat.*"));
/// bus.add_standing_order(StandingOrder::new("greet-chat", "chat", "system.hello", json!({})));
///
/// let ev = Event::new("chat.message", "iam", json!({"text": "Hi!"})).unwrap();
/// let result = bus.publish(BusMessage::fire(ev)).await;
/// ```
#[derive(Default)]
pub struct MessageBus {
    router:         Router,
    subscriptions:  SubscriptionManager,
    standing_orders: StandingOrdersEngine,
    config:         RoutingConfig,
}

impl MessageBus {
    /// Create an empty bus.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a bus pre-loaded with routing config.
    pub fn with_config(config: RoutingConfig) -> Self {
        Self { config, ..Default::default() }
    }

    // ── Configuration ──────────────────────────────────────────────────────────

    /// Load routing rules from a TOML string. Replaces existing config.
    pub fn load_config_toml(&mut self, toml: &str) -> Result<(), String> {
        self.config = RoutingConfig::from_toml(toml)?;
        Ok(())
    }

    /// Load routing rules from a file. Replaces existing config.
    pub fn load_config_file(&mut self, path: &str) -> Result<(), String> {
        self.config = RoutingConfig::load(path)?;
        Ok(())
    }

    // ── Subscriptions ──────────────────────────────────────────────────────────

    /// Register a subscription. Returns the stored entry (with generated ID).
    pub fn subscribe(&mut self, sub: Subscription) -> Subscription {
        self.subscriptions.add(sub)
    }

    /// Remove a subscription by ID. Returns `true` if it existed.
    pub fn unsubscribe(&mut self, id: uuid::Uuid) -> bool {
        self.subscriptions.remove(id)
    }

    /// Returns all subscriptions for a given role.
    pub fn subscriptions_for_role(&self, role: &str) -> Vec<&Subscription> {
        self.subscriptions.for_role(role)
    }

    // ── Standing Orders ────────────────────────────────────────────────────────

    /// Register a standing order.
    pub fn add_standing_order(&mut self, order: StandingOrder) {
        self.standing_orders.add(order);
    }

    /// Remove a standing order by ID.
    pub fn remove_standing_order(&mut self, id: uuid::Uuid) -> bool {
        self.standing_orders.remove(id)
    }

    /// Fire all standing orders triggered by `role` appearing on the bus.
    ///
    /// Returns the generated events (ready to publish). Standing order events
    /// are attributed to `"fsn-bus"` as the source.
    pub fn trigger_role(&self, role: &str) -> Vec<Result<Event, BusError>> {
        self.standing_orders.trigger_for_role(role, "fsn-bus")
    }

    // ── Router / Handlers ──────────────────────────────────────────────────────

    /// Register a custom [`TopicHandler`] (e.g. a [`BusBridge`](crate::bridge::BusBridge)).
    pub fn add_handler(&mut self, handler: Arc<dyn TopicHandler>) {
        self.router.register(handler);
    }

    // ── Publishing ─────────────────────────────────────────────────────────────

    /// Publish a [`BusMessage`] to all matching subscribers and handlers.
    ///
    /// Steps:
    /// 1. Resolve delivery + storage from the routing config (config may override
    ///    the message's explicit values when the config has a higher-priority match).
    /// 2. Dispatch to all registered [`TopicHandler`]s via the router.
    /// 3. Collect the matching subscription roles for the return value.
    pub async fn publish(&self, msg: BusMessage) -> PublishedEvent {
        let topic = msg.event.topic().to_string();
        let source_role = msg.event.meta.source.clone();

        // Resolve final delivery + storage from config (config wins over message default).
        let resolved_delivery = self.config.delivery_for(&topic, Some(&source_role));
        let resolved_storage  = self.config.storage_for(&topic, Some(&source_role));

        // Dispatch to router (handles, bridges, etc.).
        let handler_results = self.router.dispatch(&msg.event).await;

        // Collect which roles receive this event (matching subscriptions).
        let delivered_to: Vec<String> = self.subscriptions
            .matching(&topic, None)
            .into_iter()
            .map(|s| s.subscriber_role.clone())
            .collect();

        PublishedEvent {
            delivery: resolved_delivery,
            storage:  resolved_storage,
            delivered_to,
            handler_results,
            message: msg,
        }
    }

    // ── Iteration ─────────────────────────────────────────────────────────────

    /// Iterate over all active subscriptions.
    pub fn subscriptions_iter(&self) -> impl Iterator<Item = &Subscription> {
        self.subscriptions.iter()
    }

    /// Iterate over all registered standing orders.
    pub fn standing_orders_iter(&self) -> impl Iterator<Item = &StandingOrder> {
        self.standing_orders.iter()
    }

    // ── Diagnostics ───────────────────────────────────────────────────────────

    /// Number of active subscriptions.
    pub fn subscription_count(&self) -> usize {
        self.subscriptions.len()
    }

    /// Number of registered standing orders.
    pub fn standing_order_count(&self) -> usize {
        self.standing_orders.len()
    }

    /// Number of registered route handlers.
    pub fn handler_count(&self) -> usize {
        self.router.handler_count()
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::message::BusMessage;
    use crate::standing_order::StandingOrder;
    use crate::subscription::Subscription;
    use serde_json::json;

    #[tokio::test]
    async fn publish_delivers_to_subscriptions() {
        let mut bus = MessageBus::new();
        bus.subscribe(Subscription::new("chat", "chat.*"));
        bus.subscribe(Subscription::new("iam", "auth.*"));

        let ev = Event::new("chat.message", "chat-svc", json!({})).unwrap();
        let result = bus.publish(BusMessage::fire(ev)).await;

        assert_eq!(result.delivered_to, vec!["chat".to_string()]);
    }

    #[tokio::test]
    async fn routing_config_overrides_delivery() {
        let toml = r#"
[[rules]]
name          = "auth"
topic_pattern = "auth.*"
delivery      = "guaranteed"
storage       = "until-ack"
priority      = 10
"#;
        let mut bus = MessageBus::new();
        bus.load_config_toml(toml).unwrap();

        let ev = Event::new("auth.login", "iam", json!({})).unwrap();
        let msg = BusMessage::fire(ev); // fire-and-forget by default
        let result = bus.publish(msg).await;

        assert_eq!(result.delivery, DeliveryType::Guaranteed);
        assert_eq!(result.storage, StorageType::UntilAck);
    }

    #[test]
    fn trigger_role_fires_standing_orders() {
        let mut bus = MessageBus::new();
        bus.add_standing_order(StandingOrder::new(
            "greet-chat",
            "chat",
            "system.hello",
            json!({}),
        ));

        let events = bus.trigger_role("chat");
        assert_eq!(events.len(), 1);
        let ev = events[0].as_ref().unwrap();
        assert_eq!(ev.topic(), "system.hello");
    }

    #[test]
    fn subscription_lifecycle() {
        let mut bus = MessageBus::new();
        let sub = bus.subscribe(Subscription::new("chat", "#"));
        assert_eq!(bus.subscription_count(), 1);
        assert!(bus.unsubscribe(sub.id));
        assert_eq!(bus.subscription_count(), 0);
    }
}
