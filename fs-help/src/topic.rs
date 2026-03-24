// Help topic definition.

use std::sync::Arc;

use crate::kind::{HelpKindArc, InternalHelp};

// ── HelpTopic ─────────────────────────────────────────────────────────────────

/// A single help topic.
///
/// `title_key` and `content_key` are i18n keys resolved at render time.
/// All IDs and keywords are ASCII-lowercase for case-insensitive search.
///
/// The `kind` field carries the strategy for this topic (external vs internal),
/// which determines which links and search queries are available.
#[derive(Debug, Clone)]
pub struct HelpTopic {
    /// Unique identifier, e.g. `"project.create"`.
    pub id: String,
    /// i18n key for the short title shown in lists.
    pub title_key: String,
    /// i18n key for the multi-line help body.
    pub content_key: String,
    /// IDs of related topics shown at the bottom of the help panel.
    pub related: Vec<String>,
    /// Search keywords (lowercase) for `HelpSystem::search()`.
    pub keywords: Vec<String>,
    /// Strategy: external (3rd-party) or internal (FreeSynergy) help.
    /// Carries links (website, docs, git) and an optional search query.
    pub kind: HelpKindArc,
}

impl HelpTopic {
    /// Construct a minimal help topic.
    /// Defaults to `InternalHelp` with no links — use `with_kind()` to override.
    pub fn new(
        id: impl Into<String>,
        title_key: impl Into<String>,
        content_key: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            title_key: title_key.into(),
            content_key: content_key.into(),
            related: Vec::new(),
            keywords: Vec::new(),
            kind: Arc::new(InternalHelp::new()),
        }
    }

    /// Builder: add related topic IDs.
    pub fn related(mut self, ids: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.related = ids.into_iter().map(Into::into).collect();
        self
    }

    /// Builder: add search keywords (stored lowercase).
    pub fn keywords(mut self, words: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.keywords = words.into_iter().map(|w| w.into().to_lowercase()).collect();
        self
    }

    /// Builder: set the help kind strategy (external or internal).
    pub fn with_kind(mut self, kind: HelpKindArc) -> Self {
        self.kind = kind;
        self
    }

    /// Convenience accessor: links from the kind strategy.
    pub fn links(&self) -> &[crate::link::HelpLink] {
        self.kind.links()
    }

    /// Convenience accessor: search query from the kind strategy.
    pub fn search_query(&self) -> Option<&str> {
        self.kind.search_query()
    }
}
