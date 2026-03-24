// fs-help/link.rs — Typed external resource links shown in the help panel.

use serde::{Deserialize, Serialize};

// ── HelpLinkKind ──────────────────────────────────────────────────────────────

/// The kind of external resource a [`HelpLink`] points to.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HelpLinkKind {
    /// Project or product website.
    Website,
    /// Documentation site.
    Docs,
    /// Source code repository (Git).
    Git,
}

impl HelpLinkKind {
    /// Default i18n key for this link kind.
    /// Used when no custom `label_key` is set on the [`HelpLink`].
    pub fn default_label_key(&self) -> &'static str {
        match self {
            Self::Website => "help.link.website",
            Self::Docs => "help.link.docs",
            Self::Git => "help.link.git",
        }
    }

    /// Icon character for this link kind (displayed in the help panel).
    pub fn icon(&self) -> &'static str {
        match self {
            Self::Website => "🌐",
            Self::Docs => "📖",
            Self::Git => "⑂",
        }
    }
}

// ── HelpLink ──────────────────────────────────────────────────────────────────

/// A typed link to an external resource shown in the help panel.
///
/// `label_key` is an i18n key resolved at render time.
/// If empty, the renderer falls back to [`HelpLinkKind::default_label_key`].
///
/// # Example (TOML in help/overview.toml)
///
/// ```toml
/// [[topic.link]]
/// kind = "website"
/// url  = "https://kanidm.com"
///
/// [[topic.link]]
/// kind      = "git"
/// url       = "https://github.com/kanidm/kanidm"
/// label_key = "help.link.git.kanidm"
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HelpLink {
    /// Kind of resource.
    pub kind: HelpLinkKind,

    /// i18n key for the link label; falls back to `kind.default_label_key()` if empty.
    #[serde(default)]
    pub label_key: String,

    /// Target URL.
    pub url: String,
}

impl HelpLink {
    /// Create a link with no custom label key (uses `kind.default_label_key()`).
    pub fn new(kind: HelpLinkKind, url: impl Into<String>) -> Self {
        Self {
            kind,
            label_key: String::new(),
            url: url.into(),
        }
    }

    /// Builder: override the default label key.
    pub fn with_label_key(mut self, key: impl Into<String>) -> Self {
        self.label_key = key.into();
        self
    }

    /// The effective i18n key: custom key if set, otherwise the kind's default.
    pub fn effective_label_key(&self) -> &str {
        if self.label_key.is_empty() {
            self.kind.default_label_key()
        } else {
            &self.label_key
        }
    }

    // ── Convenience constructors ──────────────────────────────────────────────

    pub fn website(url: impl Into<String>) -> Self {
        Self::new(HelpLinkKind::Website, url)
    }

    pub fn docs(url: impl Into<String>) -> Self {
        Self::new(HelpLinkKind::Docs, url)
    }

    pub fn git(url: impl Into<String>) -> Self {
        Self::new(HelpLinkKind::Git, url)
    }
}
