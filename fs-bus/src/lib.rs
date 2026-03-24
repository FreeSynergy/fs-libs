//! Async event bus for FreeSynergy.
//!
//! Provides topic-based event routing, buffered delivery with exponential-backoff
//! retry, role-based subscriptions, standing orders, configurable routing rules,
//! and optional bus-to-bus bridging.
//!
//! # Architecture
//!
//! ```text
//! Producer → BusMessage → MessageBus → Router → [TopicHandler, BusBridge, …]
//!                                    ↓
//!                          SubscriptionManager (role → topic filter)
//!                                    ↓
//!                          StandingOrdersEngine (persistent triggers)
//!                                    ↓
//!                          RoutingConfig (TOML rules → delivery + storage)
//! ```

#[cfg(feature = "bridge")]
pub mod bridge;
pub mod buffer;
pub mod config;
pub mod error;
pub mod event;
pub mod message;
pub mod message_bus;
pub mod router;
pub mod standing_order;
pub mod subscription;
pub mod topic;
pub mod transform;

// ── Flat re-exports ───────────────────────────────────────────────────────────

#[cfg(feature = "bridge")]
pub use bridge::{ArcBusBridge, BusBridge, BusBridgeConfig};
pub use buffer::{EventBuffer, RetryPolicy};
pub use config::{RoutingConfig, RoutingRule};
pub use error::BusError;
pub use event::{Event, EventId, EventMeta};
pub use message::{BusMessage, DeliveryType, StorageType};
pub use message_bus::{MessageBus, PublishedEvent};
pub use router::Router;
pub use standing_order::{StandingOrder, StandingOrdersEngine};
pub use subscription::{Subscription, SubscriptionManager};
pub use topic::{topic_matches, TopicHandler};
pub use transform::{ChainTransform, Transform};

#[cfg(feature = "tera-transform")]
pub use transform::TeraTransform;
