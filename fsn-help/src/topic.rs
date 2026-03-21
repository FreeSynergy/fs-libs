// Help topic definition.

// ── HelpTopic ─────────────────────────────────────────────────────────────────

/// A single help topic.
///
/// `title_key` and `content_key` are i18n keys resolved at render time.
/// All IDs and keywords are ASCII-lowercase for case-insensitive search.
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
}

impl HelpTopic {
    /// Construct a minimal help topic with no related topics or keywords.
    pub fn new(
        id:          impl Into<String>,
        title_key:   impl Into<String>,
        content_key: impl Into<String>,
    ) -> Self {
        Self {
            id:          id.into(),
            title_key:   title_key.into(),
            content_key: content_key.into(),
            related:     Vec::new(),
            keywords:    Vec::new(),
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
}
