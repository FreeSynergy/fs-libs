//! Core resource abstraction — the base of all managed FreeSynergy entities.

use serde::{Deserialize, Serialize};

// ── ResourceKind ──────────────────────────────────────────────────────────────

/// All top-level resource categories managed by FreeSynergy tools.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResourceKind {
    /// A FreeSynergy deployment project (group of hosts + services).
    Project,
    /// A physical or virtual machine managed by this node.
    Host,
    /// A running containerized service (e.g. Kanidm, Stalwart).
    Service,
    /// An installable module from the store.
    Plugin,
    /// A UI/TUI theme package.
    Theme,
}

impl ResourceKind {
    /// Human-readable singular label for UI display.
    pub fn label(self) -> &'static str {
        match self {
            ResourceKind::Project => "Project",
            ResourceKind::Host    => "Host",
            ResourceKind::Service => "Service",
            ResourceKind::Plugin  => "Plugin",
            ResourceKind::Theme   => "Theme",
        }
    }

    /// i18n key for the resource kind label.
    pub fn i18n_key(self) -> &'static str {
        match self {
            ResourceKind::Project => "noun.project",
            ResourceKind::Host    => "noun.host",
            ResourceKind::Service => "noun.service",
            ResourceKind::Plugin  => "noun.plugin",
            ResourceKind::Theme   => "noun.theme",
        }
    }
}

// ── Resource trait ────────────────────────────────────────────────────────────

/// Base trait for any entity managed by FreeSynergy tools (Project, Host, Service …).
///
/// Implement this so that generic UI code (lists, detail panels, health checks)
/// can handle all managed entity types uniformly without downcasting.
pub trait Resource: Send + Sync {
    /// Unique stable identifier (slug), e.g. `"my-project"` or `"vps-01"`.
    fn id(&self) -> &str;

    /// Which category this resource belongs to.
    fn kind(&self) -> ResourceKind;

    /// Human-readable name shown in list rows and panel headers.
    fn display_name(&self) -> &str;
}

// ── Meta ──────────────────────────────────────────────────────────────────────

/// Generic metadata container — common fields shared by all resources.
///
/// Intended to be embedded in concrete resource structs so that generic
/// code can read `id`, `name`, and `tags` without knowing the concrete type.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Meta {
    /// Unique slug identifier (URL-safe, lowercase, hyphens).
    pub id: String,
    /// Human-readable display name.
    pub name: String,
    /// Optional multi-line description shown in detail panels.
    pub description: Option<String>,
    /// Arbitrary tags used for filtering in the store / UI.
    pub tags: Vec<String>,
    /// ISO-8601 creation timestamp (set by the backend, optional in config).
    pub created_at: Option<String>,
    /// ISO-8601 last-update timestamp.
    pub updated_at: Option<String>,
}

impl Meta {
    /// Create a `Meta` with `id` and `name` set; all other fields default.
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            ..Default::default()
        }
    }

    /// `true` when the `id` is a non-empty, non-whitespace string.
    pub fn is_valid(&self) -> bool {
        !self.id.trim().is_empty()
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resource_kind_label() {
        assert_eq!(ResourceKind::Host.label(), "Host");
        assert_eq!(ResourceKind::Service.label(), "Service");
    }

    #[test]
    fn resource_kind_i18n_key() {
        assert_eq!(ResourceKind::Project.i18n_key(), "noun.project");
    }

    #[test]
    fn meta_valid_with_non_empty_id() {
        let m = Meta::new("my-proj", "My Project");
        assert!(m.is_valid());
    }

    #[test]
    fn meta_invalid_with_whitespace_id() {
        let m = Meta { id: "  ".to_string(), ..Default::default() };
        assert!(!m.is_valid());
    }

    #[test]
    fn meta_default_tags_empty() {
        let m = Meta::new("x", "X");
        assert!(m.tags.is_empty());
    }
}
