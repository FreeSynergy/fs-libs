// fs-bus/src/message.rs — Message envelope types (delivery + storage policy).

use serde::{Deserialize, Serialize};

use crate::event::Event;

// ── DeliveryType ──────────────────────────────────────────────────────────────

/// How the bus delivers a message to subscribers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum DeliveryType {
    /// Send-and-forget: no acknowledgment required. Default.
    #[default]
    FireAndForget,
    /// Deliver until explicitly acknowledged by the subscriber.
    Guaranteed,
    /// A standing instruction that is re-executed whenever its trigger fires.
    StandingOrder,
}

impl DeliveryType {
    /// String form used in DB and config files.
    pub fn as_str(&self) -> &str {
        match self {
            Self::FireAndForget  => "fire-and-forget",
            Self::Guaranteed     => "guaranteed",
            Self::StandingOrder  => "standing-order",
        }
    }
}

impl std::fmt::Display for DeliveryType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

// ── StorageType ───────────────────────────────────────────────────────────────

/// How the bus persists a message after delivery.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum StorageType {
    /// Do not persist the event. Default.
    #[default]
    NoStore,
    /// Persist until the subscriber acknowledges receipt.
    UntilAck,
    /// Persist indefinitely (audit log, compliance).
    Persistent,
}

impl StorageType {
    /// String form used in DB and config files.
    pub fn as_str(&self) -> &str {
        match self {
            Self::NoStore    => "no-store",
            Self::UntilAck   => "until-ack",
            Self::Persistent => "persistent",
        }
    }
}

impl std::fmt::Display for StorageType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

// ── BusMessage ────────────────────────────────────────────────────────────────

/// A bus message: an [`Event`] wrapped with delivery and storage policies.
///
/// The bus routes the inner event to all matching subscribers according to
/// the policies. Callers that just want fire-and-forget can use the shorthand
/// [`BusMessage::fire`] constructor.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BusMessage {
    /// The event to deliver.
    pub event: Event,
    /// How the message is delivered (default: [`DeliveryType::FireAndForget`]).
    pub delivery: DeliveryType,
    /// How the message is stored after delivery (default: [`StorageType::NoStore`]).
    pub storage: StorageType,
}

impl BusMessage {
    /// Build a message with explicit delivery and storage policies.
    pub fn new(event: Event, delivery: DeliveryType, storage: StorageType) -> Self {
        Self { event, delivery, storage }
    }

    /// Fire-and-forget convenience constructor (no storage).
    pub fn fire(event: Event) -> Self {
        Self { event, delivery: DeliveryType::FireAndForget, storage: StorageType::NoStore }
    }

    /// Guaranteed delivery with until-ack storage.
    pub fn guaranteed(event: Event) -> Self {
        Self { event, delivery: DeliveryType::Guaranteed, storage: StorageType::UntilAck }
    }

    /// Persistent standing order.
    pub fn standing(event: Event) -> Self {
        Self { event, delivery: DeliveryType::StandingOrder, storage: StorageType::Persistent }
    }

    /// Return the topic of the inner event.
    pub fn topic(&self) -> &str {
        self.event.topic()
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::Event;

    #[test]
    fn delivery_type_display() {
        assert_eq!(DeliveryType::FireAndForget.as_str(), "fire-and-forget");
        assert_eq!(DeliveryType::Guaranteed.as_str(), "guaranteed");
        assert_eq!(DeliveryType::StandingOrder.as_str(), "standing-order");
    }

    #[test]
    fn storage_type_display() {
        assert_eq!(StorageType::NoStore.as_str(), "no-store");
        assert_eq!(StorageType::UntilAck.as_str(), "until-ack");
        assert_eq!(StorageType::Persistent.as_str(), "persistent");
    }

    #[test]
    fn fire_constructor() {
        let ev = Event::new("test.topic", "test", ()).unwrap();
        let msg = BusMessage::fire(ev);
        assert_eq!(msg.delivery, DeliveryType::FireAndForget);
        assert_eq!(msg.storage, StorageType::NoStore);
        assert_eq!(msg.topic(), "test.topic");
    }

    #[test]
    fn guaranteed_constructor() {
        let ev = Event::new("test.topic", "test", ()).unwrap();
        let msg = BusMessage::guaranteed(ev);
        assert_eq!(msg.delivery, DeliveryType::Guaranteed);
        assert_eq!(msg.storage, StorageType::UntilAck);
    }
}
