// fs-bus/src/buffer.rs — Buffered delivery with configurable retry policy.

use std::collections::VecDeque;

use backon::{ExponentialBuilder, Retryable};
use tracing::{info, warn};

use crate::error::BusError;
use crate::event::Event;
use crate::router::Router;

// ── RetryPolicy ───────────────────────────────────────────────────────────────

/// Exponential backoff configuration for event re-delivery.
#[derive(Debug, Clone)]
pub struct RetryPolicy {
    /// Maximum number of delivery attempts (including the first).
    pub max_attempts: u32,
    /// Initial delay between attempts in milliseconds.
    pub base_delay_ms: u64,
    /// Maximum delay between attempts in milliseconds.
    pub max_delay_ms: u64,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            base_delay_ms: 100,
            max_delay_ms: 5_000,
        }
    }
}

// ── EventBuffer ───────────────────────────────────────────────────────────────

/// Buffers events and flushes them to a [`Router`] with automatic retry.
///
/// Events that fail after all retry attempts are passed to an `on_failed`
/// callback instead of being silently dropped.
///
/// # Example
///
/// ```rust,ignore
/// use fs_bus::{EventBuffer, RetryPolicy, Router, Event};
///
/// let mut buf = EventBuffer::new(RetryPolicy::default());
/// buf.push(event);
/// buf.flush(&router, |ev, err| {
///     eprintln!("failed: {} — {}", ev.topic(), err);
/// }).await;
/// ```
pub struct EventBuffer {
    queue: VecDeque<Event>,
    policy: RetryPolicy,
}

impl EventBuffer {
    /// Create a new buffer with the given retry policy.
    pub fn new(policy: RetryPolicy) -> Self {
        Self {
            queue: VecDeque::new(),
            policy,
        }
    }

    /// Create a buffer with the default retry policy.
    pub fn with_defaults() -> Self {
        Self::new(RetryPolicy::default())
    }

    /// Add an event to the back of the buffer.
    pub fn push(&mut self, event: Event) {
        self.queue.push_back(event);
    }

    /// Number of events currently buffered.
    pub fn len(&self) -> usize {
        self.queue.len()
    }

    /// `true` when the buffer is empty.
    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }

    /// Flush all buffered events through `router`.
    ///
    /// Each event is dispatched with exponential backoff. If a dispatch produces
    /// at least one non-error result it is considered delivered. Events that
    /// exhaust all retry attempts are passed to `on_failed`.
    ///
    /// The buffer is empty after this call regardless of delivery outcome.
    pub async fn flush<F>(&mut self, router: &Router, mut on_failed: F)
    where
        F: FnMut(Event, BusError),
    {
        while let Some(event) = self.queue.pop_front() {
            let max = self.policy.max_attempts;
            let base = self.policy.base_delay_ms;
            let cap = self.policy.max_delay_ms;

            let result = Self::try_deliver(&event, router, max, base, cap).await;

            match result {
                Ok(()) => info!(topic = %event.topic(), "event delivered"),
                Err(e) => {
                    warn!(topic = %event.topic(), error = %e, "event delivery failed permanently");
                    on_failed(event, e);
                }
            }
        }
    }

    async fn try_deliver(
        event: &Event,
        router: &Router,
        max_attempts: u32,
        base_delay_ms: u64,
        max_delay_ms: u64,
    ) -> Result<(), BusError> {
        let backoff = ExponentialBuilder::default()
            .with_min_delay(std::time::Duration::from_millis(base_delay_ms))
            .with_max_delay(std::time::Duration::from_millis(max_delay_ms))
            .with_max_times(max_attempts as usize);

        let op = || async {
            let results = router.dispatch(event).await;
            // Succeed if there are no errors (or no handlers at all).
            let errors: Vec<_> = results.into_iter().filter_map(|r| r.err()).collect();
            if errors.is_empty() {
                Ok(())
            } else {
                Err(BusError::handler(
                    event.topic(),
                    errors
                        .iter()
                        .map(|e| e.to_string())
                        .collect::<Vec<_>>()
                        .join("; "),
                ))
            }
        };

        op.retry(backoff).await
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::Event;
    use crate::topic::TopicHandler;
    use async_trait::async_trait;
    use std::sync::{
        atomic::{AtomicU32, Ordering},
        Arc,
    };

    struct OkHandler(Arc<AtomicU32>);

    #[async_trait]
    impl TopicHandler for OkHandler {
        fn topic_pattern(&self) -> &str {
            "#"
        }
        async fn handle(&self, _: &Event) -> Result<(), BusError> {
            self.0.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }
    }

    #[tokio::test]
    async fn flush_delivers_events() {
        let count = Arc::new(AtomicU32::new(0));
        let mut router = Router::new();
        router.register(Arc::new(OkHandler(count.clone())));

        let mut buf = EventBuffer::with_defaults();
        buf.push(Event::new("test.a", "test", ()).unwrap());
        buf.push(Event::new("test.b", "test", ()).unwrap());

        let mut failed = 0u32;
        buf.flush(&router, |_, _| failed += 1).await;

        assert_eq!(count.load(Ordering::SeqCst), 2);
        assert_eq!(failed, 0);
        assert!(buf.is_empty());
    }
}
