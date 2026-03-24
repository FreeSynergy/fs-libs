// fs-help/kind.rs — HelpKind strategy trait with ExternalHelp + InternalHelp.
//
// Design: Strategy Pattern.
//
// `HelpKind` is a trait with two concrete implementations:
//   - ExternalHelp — 3rd-party programs: website, git, optional tutorial search query
//   - InternalHelp — FreeSynergy programs: links come from the package's own origin
//
// Callers use `topic.kind().links()` / `topic.kind().search_query()` without
// any match blocks. New help strategies can be added without touching callers.

use std::sync::Arc;

use crate::link::HelpLink;

// ── HelpKind ──────────────────────────────────────────────────────────────────

/// Strategy trait distinguishing different kinds of help content.
///
/// Implementations carry the data specific to their kind. Callers always
/// go through the trait — no `match` on the concrete type.
pub trait HelpKind: Send + Sync + std::fmt::Debug {
    /// External resource links (website, docs, git repo).
    fn links(&self) -> &[HelpLink];

    /// Engine-agnostic tutorial search query, e.g. `"kanidm identity provider tutorial"`.
    ///
    /// Returns `None` for internal help (use the docs link instead).
    fn search_query(&self) -> Option<&str>;

    /// Stable identifier for serialization / logging (`"external"` | `"internal"`).
    fn kind_name(&self) -> &'static str;
}

// ── ExternalHelp ──────────────────────────────────────────────────────────────

/// Help for 3rd-party programs not developed by FreeSynergy.
///
/// Carries: links to the upstream project (website, git, docs) and an optional
/// tutorial search query. The textual description lives in the package's Fluent
/// file (`help/{lang}/overview.ftl`).
#[derive(Debug, Clone, Default)]
pub struct ExternalHelp {
    pub links: Vec<HelpLink>,
    pub search_query: Option<String>,
}

impl ExternalHelp {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_links(mut self, links: Vec<HelpLink>) -> Self {
        self.links = links;
        self
    }

    pub fn with_search(mut self, query: impl Into<String>) -> Self {
        self.search_query = Some(query.into());
        self
    }

    pub fn with_search_opt(mut self, query: Option<String>) -> Self {
        self.search_query = query;
        self
    }
}

impl HelpKind for ExternalHelp {
    fn links(&self) -> &[HelpLink] {
        &self.links
    }
    fn search_query(&self) -> Option<&str> {
        self.search_query.as_deref()
    }
    fn kind_name(&self) -> &'static str {
        "external"
    }
}

// ── InternalHelp ──────────────────────────────────────────────────────────────

/// Help for FreeSynergy's own programs.
///
/// Links come from the package's own origin metadata (website, docs, git).
/// No tutorial search query — the package's own docs are the reference.
#[derive(Debug, Clone, Default)]
pub struct InternalHelp {
    pub links: Vec<HelpLink>,
}

impl InternalHelp {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_links(mut self, links: Vec<HelpLink>) -> Self {
        self.links = links;
        self
    }
}

impl HelpKind for InternalHelp {
    fn links(&self) -> &[HelpLink] {
        &self.links
    }
    fn search_query(&self) -> Option<&str> {
        None
    }
    fn kind_name(&self) -> &'static str {
        "internal"
    }
}

// ── HelpKindArc ───────────────────────────────────────────────────────────────

/// Shared-ownership handle to any [`HelpKind`] implementation.
pub type HelpKindArc = Arc<dyn HelpKind + Send + Sync>;

/// Wrap an [`ExternalHelp`] in an [`Arc`] for use in [`HelpTopic`].
pub fn external(help: ExternalHelp) -> HelpKindArc {
    Arc::new(help)
}

/// Wrap an [`InternalHelp`] in an [`Arc`] for use in [`HelpTopic`].
pub fn internal(help: InternalHelp) -> HelpKindArc {
    Arc::new(help)
}
