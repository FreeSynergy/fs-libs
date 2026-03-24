// Context-based help lookup with hierarchical fallback.

use std::collections::HashMap;

use crate::topic::HelpTopic;

// ── HelpContext ───────────────────────────────────────────────────────────────

/// Context-sensitive help lookup.
///
/// Walks up the dot-separated hierarchy from most specific to least specific:
///   1. Exact match `"project.create.host"`
///   2. Parent `"project.create"`
///   3. Root `"project"`
pub struct HelpContext<'a> {
    topics: &'a HashMap<String, HelpTopic>,
}

impl<'a> HelpContext<'a> {
    /// Create a context lookup over `topics`.
    pub fn new(topics: &'a HashMap<String, HelpTopic>) -> Self {
        Self { topics }
    }

    /// Find the best matching topic for `ctx`.
    ///
    /// Returns `None` if no topic matches even the root segment.
    pub fn lookup(&self, ctx: &str) -> Option<&'a HelpTopic> {
        // Exact match
        if let Some(t) = self.topics.get(ctx) {
            return Some(t);
        }

        // Walk up the dot-separated hierarchy
        let mut parts: Vec<&str> = ctx.split('.').collect();
        while parts.len() > 1 {
            parts.pop();
            let candidate = parts.join(".");
            if let Some(t) = self.topics.get(candidate.as_str()) {
                return Some(t);
            }
        }
        None
    }
}
