// fs-bus/src/topic.rs — TopicHandler trait and glob pattern matching.

use async_trait::async_trait;

use crate::error::BusError;
use crate::event::Event;

// ── TopicHandler ──────────────────────────────────────────────────────────────

/// A subscriber that handles events matching a topic pattern.
///
/// Implement this trait on any type that wants to receive bus events.
/// The [`Router`](crate::router::Router) calls [`handle`](TopicHandler::handle)
/// for every event whose topic matches [`topic_pattern`](TopicHandler::topic_pattern).
///
/// # Pattern syntax
///
/// - `"deploy.started"` — exact match
/// - `"deploy.*"` — matches one segment wildcard (`deploy.started`, `deploy.failed`)
/// - `"*"` — matches any single-segment topic
/// - `"#"` — matches any topic (greedy, any number of segments)
///
/// # Example
///
/// ```rust,ignore
/// use fs_bus::{TopicHandler, Event, BusError};
/// use async_trait::async_trait;
///
/// struct DeployLogger;
///
/// #[async_trait]
/// impl TopicHandler for DeployLogger {
///     fn topic_pattern(&self) -> &str { "deploy.*" }
///
///     async fn handle(&self, event: &Event) -> Result<(), BusError> {
///         println!("deploy event: {}", event.topic());
///         Ok(())
///     }
/// }
/// ```
#[async_trait]
pub trait TopicHandler: Send + Sync {
    /// Glob-style pattern this handler subscribes to.
    fn topic_pattern(&self) -> &str;

    /// Process an event that matched this handler's pattern.
    async fn handle(&self, event: &Event) -> Result<(), BusError>;
}

// ── Pattern matching ──────────────────────────────────────────────────────────

/// Check whether `pattern` matches `topic`.
///
/// - `#` in `pattern` matches any number of dot-separated segments.
/// - `*` matches exactly one segment (no dots).
/// - Anything else must equal the corresponding segment literally.
pub fn topic_matches(pattern: &str, topic: &str) -> bool {
    if pattern == "#" {
        return true;
    }
    let mut pat_parts = pattern.split('.');
    let mut top_parts = topic.split('.');

    loop {
        match (pat_parts.next(), top_parts.next()) {
            (Some("#"), _) => return true,
            (Some("*"), Some(_)) => continue,
            (Some(p), Some(t)) if p == t => continue,
            (None, None) => return true,
            _ => return false,
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_match() {
        assert!(topic_matches("deploy.started", "deploy.started"));
        assert!(!topic_matches("deploy.started", "deploy.failed"));
    }

    #[test]
    fn single_wildcard() {
        assert!(topic_matches("deploy.*", "deploy.started"));
        assert!(topic_matches("deploy.*", "deploy.failed"));
        assert!(!topic_matches("deploy.*", "deploy.started.now"));
    }

    #[test]
    fn greedy_wildcard() {
        assert!(topic_matches("#", "deploy.started"));
        assert!(topic_matches("#", "anything.at.all"));
        assert!(topic_matches("deploy.#", "deploy.started.now"));
    }

    #[test]
    fn no_match() {
        assert!(!topic_matches("health.*", "deploy.started"));
    }
}
