// fsn-bus/src/router.rs — Event dispatcher that routes events to handlers.

use std::sync::Arc;

use tracing::{debug, warn};

use crate::error::BusError;
use crate::event::Event;
use crate::topic::{TopicHandler, topic_matches};

// ── Router ────────────────────────────────────────────────────────────────────

/// Routes events to registered [`TopicHandler`]s by glob-pattern matching.
///
/// Handlers are matched by [`TopicHandler::topic_pattern`] against the event
/// topic. All matching handlers are called; results are collected and returned.
///
/// # Example
///
/// ```rust,ignore
/// use fsn_bus::{Router, Event};
///
/// let mut router = Router::new();
/// router.register(Arc::new(my_handler));
/// let results = router.dispatch(&event).await;
/// ```
#[derive(Default)]
pub struct Router {
    handlers: Vec<Arc<dyn TopicHandler>>,
}

impl Router {
    /// Create an empty router.
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a handler. It will receive all events matching its pattern.
    pub fn register(&mut self, handler: Arc<dyn TopicHandler>) {
        self.handlers.push(handler);
    }

    /// Dispatch `event` to all matching handlers.
    ///
    /// Returns one result per matching handler in registration order.
    /// A handler error does **not** stop other handlers from running.
    pub async fn dispatch(&self, event: &Event) -> Vec<Result<(), BusError>> {
        let mut results = Vec::new();
        for handler in &self.handlers {
            if topic_matches(handler.topic_pattern(), event.topic()) {
                debug!(
                    topic = %event.topic(),
                    pattern = %handler.topic_pattern(),
                    "dispatching event"
                );
                let result = handler.handle(event).await;
                if let Err(ref e) = result {
                    warn!(
                        topic = %event.topic(),
                        error = %e,
                        "handler returned error"
                    );
                }
                results.push(result);
            }
        }
        results
    }

    /// Return the number of registered handlers.
    pub fn handler_count(&self) -> usize {
        self.handlers.len()
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::Event;
    use async_trait::async_trait;
    use std::sync::atomic::{AtomicU32, Ordering};

    struct Counter {
        pattern: &'static str,
        count: Arc<AtomicU32>,
    }

    #[async_trait]
    impl TopicHandler for Counter {
        fn topic_pattern(&self) -> &str { self.pattern }
        async fn handle(&self, _event: &Event) -> Result<(), BusError> {
            self.count.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }
    }

    #[tokio::test]
    async fn dispatch_to_matching_handler() {
        let count = Arc::new(AtomicU32::new(0));
        let mut router = Router::new();
        router.register(Arc::new(Counter { pattern: "deploy.*", count: count.clone() }));

        let ev = Event::new("deploy.started", "test", ()).unwrap();
        let results = router.dispatch(&ev).await;
        assert_eq!(results.len(), 1);
        assert_eq!(count.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn no_dispatch_on_mismatch() {
        let count = Arc::new(AtomicU32::new(0));
        let mut router = Router::new();
        router.register(Arc::new(Counter { pattern: "health.*", count: count.clone() }));

        let ev = Event::new("deploy.started", "test", ()).unwrap();
        let results = router.dispatch(&ev).await;
        assert!(results.is_empty());
        assert_eq!(count.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn multiple_handlers_all_called() {
        let c1 = Arc::new(AtomicU32::new(0));
        let c2 = Arc::new(AtomicU32::new(0));
        let mut router = Router::new();
        router.register(Arc::new(Counter { pattern: "#", count: c1.clone() }));
        router.register(Arc::new(Counter { pattern: "deploy.*", count: c2.clone() }));

        let ev = Event::new("deploy.started", "test", ()).unwrap();
        router.dispatch(&ev).await;
        assert_eq!(c1.load(Ordering::SeqCst), 1);
        assert_eq!(c2.load(Ordering::SeqCst), 1);
    }
}
