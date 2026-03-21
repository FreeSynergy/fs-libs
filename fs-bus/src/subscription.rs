// fs-bus/src/subscription.rs — Subscription manager (role + topic + inst-tag).

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::topic::topic_matches;

// ── Subscription ──────────────────────────────────────────────────────────────

/// A single active subscription: a role listening to a topic pattern.
///
/// Subscriptions are role-based — no service is addressed directly.
/// The `inst_tag` allows distinguishing between multiple instances of the
/// same role (e.g. two chat services both providing the `chat` role).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subscription {
    /// Unique identifier for this subscription.
    pub id: Uuid,
    /// The role that receives matching events (e.g. `"chat"`, `"iam"`).
    pub subscriber_role: String,
    /// Glob-style topic filter (same syntax as [`TopicHandler`](crate::TopicHandler)).
    pub topic_filter: String,
    /// Optional instance tag — if set, only matches events from that specific instance.
    pub inst_tag: Option<String>,
    /// Whether this role has been granted read access for the matched topics.
    pub granted_read: bool,
}

impl Subscription {
    /// Build a new subscription with a generated ID.
    pub fn new(
        subscriber_role: impl Into<String>,
        topic_filter: impl Into<String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            subscriber_role: subscriber_role.into(),
            topic_filter: topic_filter.into(),
            inst_tag: None,
            granted_read: true,
        }
    }

    /// Restrict this subscription to a specific service instance.
    pub fn with_inst_tag(mut self, tag: impl Into<String>) -> Self {
        self.inst_tag = Some(tag.into());
        self
    }

    /// Revoke read access (subscription exists but events are filtered).
    pub fn deny_read(mut self) -> Self {
        self.granted_read = false;
        self
    }

    /// Returns `true` if this subscription matches `topic` and `source_inst`.
    pub fn matches(&self, topic: &str, source_inst: Option<&str>) -> bool {
        if !self.granted_read {
            return false;
        }
        if !topic_matches(&self.topic_filter, topic) {
            return false;
        }
        if let Some(tag) = &self.inst_tag {
            if source_inst != Some(tag.as_str()) {
                return false;
            }
        }
        true
    }
}

// ── SubscriptionManager ───────────────────────────────────────────────────────

/// In-memory registry of active subscriptions.
///
/// The bus calls [`SubscriptionManager::matching`] on every published event
/// to determine which roles should receive it.
///
/// For persistence, load subscriptions from the `subscriptions` table at
/// startup and call [`add`](SubscriptionManager::add) for each row.
#[derive(Debug, Default)]
pub struct SubscriptionManager {
    // keyed by Uuid for O(1) removal
    subs: HashMap<Uuid, Subscription>,
}

impl SubscriptionManager {
    /// Create an empty manager.
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a subscription. Returns a clone of the stored entry.
    pub fn add(&mut self, sub: Subscription) -> Subscription {
        let stored = sub.clone();
        self.subs.insert(sub.id, sub);
        stored
    }

    /// Remove a subscription by ID. Returns `true` if it existed.
    pub fn remove(&mut self, id: Uuid) -> bool {
        self.subs.remove(&id).is_some()
    }

    /// Returns all subscriptions whose filter matches `topic`.
    ///
    /// `source_inst` is the instance tag of the publishing service (if any).
    pub fn matching<'a>(&'a self, topic: &str, source_inst: Option<&str>) -> Vec<&'a Subscription> {
        self.subs.values()
            .filter(|s| s.matches(topic, source_inst))
            .collect()
    }

    /// Returns all subscriptions for a given role.
    pub fn for_role<'a>(&'a self, role: &str) -> Vec<&'a Subscription> {
        self.subs.values()
            .filter(|s| s.subscriber_role == role)
            .collect()
    }

    /// Total number of active subscriptions.
    pub fn len(&self) -> usize {
        self.subs.len()
    }

    /// Returns `true` if there are no subscriptions.
    pub fn is_empty(&self) -> bool {
        self.subs.is_empty()
    }

    /// Iterate over all subscriptions.
    pub fn iter(&self) -> impl Iterator<Item = &Subscription> {
        self.subs.values()
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_sub(role: &str, filter: &str) -> Subscription {
        Subscription::new(role, filter)
    }

    #[test]
    fn basic_matching() {
        let mut mgr = SubscriptionManager::new();
        mgr.add(make_sub("chat", "chat.*"));
        mgr.add(make_sub("iam", "auth.*"));

        let matched = mgr.matching("chat.message", None);
        assert_eq!(matched.len(), 1);
        assert_eq!(matched[0].subscriber_role, "chat");
    }

    #[test]
    fn inst_tag_filter() {
        let mut mgr = SubscriptionManager::new();
        mgr.add(make_sub("chat", "#").with_inst_tag("chat-primary"));

        assert_eq!(mgr.matching("any.topic", Some("chat-primary")).len(), 1);
        assert_eq!(mgr.matching("any.topic", Some("chat-secondary")).len(), 0);
        assert_eq!(mgr.matching("any.topic", None).len(), 0);
    }

    #[test]
    fn deny_read_excluded() {
        let mut mgr = SubscriptionManager::new();
        mgr.add(make_sub("chat", "#").deny_read());

        assert!(mgr.matching("any.topic", None).is_empty());
    }

    #[test]
    fn remove() {
        let mut mgr = SubscriptionManager::new();
        let sub = mgr.add(make_sub("chat", "#"));
        assert_eq!(mgr.len(), 1);
        assert!(mgr.remove(sub.id));
        assert!(mgr.is_empty());
    }

    #[test]
    fn for_role() {
        let mut mgr = SubscriptionManager::new();
        mgr.add(make_sub("chat", "chat.*"));
        mgr.add(make_sub("chat", "alert.*"));
        mgr.add(make_sub("iam", "auth.*"));

        assert_eq!(mgr.for_role("chat").len(), 2);
        assert_eq!(mgr.for_role("iam").len(), 1);
        assert_eq!(mgr.for_role("unknown").len(), 0);
    }
}
