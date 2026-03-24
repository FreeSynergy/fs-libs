// fs-help — Context-sensitive help topic system for FreeSynergy.
//
// Design:
//   Composite  — HelpSystem owns a flat map of HelpTopic
//   Strategy   — HelpKind trait (ExternalHelp / InternalHelp) per topic
//   OOP        — each topic carries its own links, search query, and i18n keys

use std::collections::HashMap;

pub mod context;
pub mod kind;
pub mod link;
pub mod loader;
pub mod search;
pub mod topic;

pub use context::HelpContext;
pub use kind::{external, internal, ExternalHelp, HelpKind, HelpKindArc, InternalHelp};
pub use link::{HelpLink, HelpLinkKind};
pub use loader::HelpLoader;
pub use search::HelpSearch;
pub use topic::HelpTopic;

// ── HelpSystem ────────────────────────────────────────────────────────────────

/// Registry of all help topics, queryable by context or full-text search.
#[derive(Debug, Default)]
pub struct HelpSystem {
    topics: HashMap<String, HelpTopic>,
}

impl HelpSystem {
    /// Create an empty help system.
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a help topic. Replaces any existing topic with the same id.
    pub fn add_topic(&mut self, topic: HelpTopic) {
        self.topics.insert(topic.id.clone(), topic);
    }

    /// Retrieve the help topic for a given context string.
    ///
    /// Lookup strategy (most specific → least specific):
    ///   1. Exact match `"project.create.host"`
    ///   2. Parent `"project.create"`
    ///   3. Root `"project"`
    pub fn help_for_context(&self, ctx: &str) -> Option<&HelpTopic> {
        HelpContext::new(&self.topics).lookup(ctx)
    }

    /// Full-text search across all topics.
    pub fn search(&self, query: &str) -> Vec<&HelpTopic> {
        HelpSearch::new(&self.topics).search(query)
    }

    /// All registered topics sorted by id.
    pub fn all_topics(&self) -> Vec<&HelpTopic> {
        let mut v: Vec<&HelpTopic> = self.topics.values().collect();
        v.sort_by_key(|t| t.id.as_str());
        v
    }

    /// Retrieve related topics for a given topic id.
    pub fn related_for(&self, id: &str) -> Vec<&HelpTopic> {
        let Some(topic) = self.topics.get(id) else {
            return vec![];
        };
        topic
            .related
            .iter()
            .filter_map(|rid| self.topics.get(rid.as_str()))
            .collect()
    }

    /// Number of registered topics.
    pub fn len(&self) -> usize {
        self.topics.len()
    }

    /// `true` when no topics are registered.
    pub fn is_empty(&self) -> bool {
        self.topics.is_empty()
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_system() -> HelpSystem {
        let mut h = HelpSystem::new();
        h.add_topic(
            HelpTopic::new("project", "help.project.title", "help.project.body")
                .keywords(["project", "slug", "name"]),
        );
        h.add_topic(
            HelpTopic::new(
                "project.create",
                "help.project.create.title",
                "help.project.create.body",
            )
            .related(["project"])
            .keywords(["create", "new", "project"]),
        );
        h
    }

    #[test]
    fn exact_context_lookup() {
        let h = sample_system();
        assert_eq!(
            h.help_for_context("project.create").unwrap().id,
            "project.create"
        );
    }

    #[test]
    fn parent_context_fallback() {
        let h = sample_system();
        assert_eq!(
            h.help_for_context("project.create.host").unwrap().id,
            "project.create"
        );
    }

    #[test]
    fn root_context_fallback() {
        let h = sample_system();
        assert_eq!(h.help_for_context("project.delete").unwrap().id, "project");
    }

    #[test]
    fn search_by_keyword() {
        let h = sample_system();
        let results = h.search("create");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "project.create");
    }

    #[test]
    fn related_for_returns_topics() {
        let h = sample_system();
        let related = h.related_for("project.create");
        assert_eq!(related.len(), 1);
        assert_eq!(related[0].id, "project");
    }

    #[test]
    fn all_topics_sorted() {
        let h = sample_system();
        let all = h.all_topics();
        assert_eq!(all[0].id, "project");
        assert_eq!(all[1].id, "project.create");
    }
}
