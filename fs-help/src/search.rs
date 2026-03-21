// Full-text keyword search over help topics.

use std::collections::HashMap;

use crate::topic::HelpTopic;

// ── HelpSearch ────────────────────────────────────────────────────────────────

/// Full-text search index over help topics.
///
/// The index is built from the topic `id` and `keywords` fields.
/// All matching is case-insensitive.
pub struct HelpSearch<'a> {
    topics: &'a HashMap<String, HelpTopic>,
}

impl<'a> HelpSearch<'a> {
    /// Create a search index over `topics`.
    pub fn new(topics: &'a HashMap<String, HelpTopic>) -> Self {
        Self { topics }
    }

    /// Search for topics matching `query`.
    ///
    /// Matches `query` (case-insensitive) against:
    ///   - topic `id`
    ///   - topic `keywords`
    ///
    /// Results are sorted by topic id for deterministic ordering.
    pub fn search(&self, query: &str) -> Vec<&'a HelpTopic> {
        let q = query.to_lowercase();
        let mut results: Vec<&HelpTopic> = self.topics.values()
            .filter(|t| {
                t.id.contains(&q) || t.keywords.iter().any(|kw| kw.contains(&q))
            })
            .collect();
        results.sort_by_key(|t| t.id.as_str());
        results
    }
}
