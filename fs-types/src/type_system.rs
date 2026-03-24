//! Type system: `ServiceType`, `ContainerPurpose`, and `TypeRegistry`.
//!
//! `TypeRegistry` is loaded from the store catalog on startup and used by the TUI,
//! health checker, and dependency resolver.

use crate::StrLabel;
use serde::{Deserialize, Serialize};

// ── ServiceType ───────────────────────────────────────────────────────────────

/// A hierarchical type identifier for a service or module, e.g. `"mail/stalwart"`.
///
/// Format: `"<category>/<implementation>"` — the separator is `/`.
/// A bare category (without `/`) is also valid for abstract type references.
///
/// This classifies **services** (running containers), not store packages.
/// For store package types, see [`crate::resources::ResourceType`].
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ServiceType(pub String);

impl ServiceType {
    /// Create from any string-like value.
    pub fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }

    /// The category part before the `/`, e.g. `"mail"` from `"mail/stalwart"`.
    pub fn category(&self) -> &str {
        self.0.split('/').next().unwrap_or(&self.0)
    }

    /// The implementation part after the `/`, if present.
    pub fn implementation(&self) -> Option<&str> {
        self.0.split_once('/').map(|(_, impl_)| impl_)
    }

    /// `true` when the type has an implementation suffix.
    pub fn is_concrete(&self) -> bool {
        self.0.contains('/')
    }

    /// The raw string value.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for ServiceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl From<&str> for ServiceType {
    fn from(s: &str) -> Self {
        Self(s.to_owned())
    }
}

// ── ContainerPurpose ──────────────────────────────────────────────────────────

/// The functional purpose of a containerized service.
///
/// Used by the capability matcher and health checker to classify what
/// a running service does, independent of the specific implementation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContainerPurpose {
    /// Reverse proxy + TLS termination (e.g. Zentinel).
    Proxy,
    /// Identity and access management / OIDC provider (e.g. Kanidm, Authentik).
    Iam,
    /// Mail transfer agent + IMAP server (e.g. Stalwart).
    Mail,
    /// Git hosting (e.g. Forgejo).
    Git,
    /// Knowledge base / wiki (e.g. Outline).
    Wiki,
    /// Real-time chat / messaging (e.g. Matrix/Tuwunel).
    Chat,
    /// Real-time collaboration / document editing (e.g. CryptPad).
    Collab,
    /// Task and project management (e.g. Vikunja).
    Tasks,
    /// Event ticketing (e.g. Pretix).
    Tickets,
    /// Maps and geolocation (e.g. uMap).
    Maps,
    /// Observability / log aggregation (e.g. OpenObserver).
    Monitoring,
    /// Relational database (e.g. PostgreSQL).
    Database,
    /// In-memory cache / key-value store (e.g. Dragonfly).
    Cache,
    /// Any purpose not covered by the above variants.
    Custom,
}

impl StrLabel for ContainerPurpose {
    fn label(&self) -> &'static str {
        match self {
            Self::Proxy => "Proxy",
            Self::Iam => "IAM",
            Self::Mail => "Mail",
            Self::Git => "Git",
            Self::Wiki => "Wiki",
            Self::Chat => "Chat",
            Self::Collab => "Collab",
            Self::Tasks => "Tasks",
            Self::Tickets => "Tickets",
            Self::Maps => "Maps",
            Self::Monitoring => "Monitoring",
            Self::Database => "Database",
            Self::Cache => "Cache",
            Self::Custom => "Custom",
        }
    }
}

crate::impl_str_label_display!(ContainerPurpose);

impl ContainerPurpose {
    /// i18n key.
    pub fn i18n_key(self) -> &'static str {
        match self {
            ContainerPurpose::Proxy => "purpose.proxy",
            ContainerPurpose::Iam => "purpose.iam",
            ContainerPurpose::Mail => "purpose.mail",
            ContainerPurpose::Git => "purpose.git",
            ContainerPurpose::Wiki => "purpose.wiki",
            ContainerPurpose::Chat => "purpose.chat",
            ContainerPurpose::Collab => "purpose.collab",
            ContainerPurpose::Tasks => "purpose.tasks",
            ContainerPurpose::Tickets => "purpose.tickets",
            ContainerPurpose::Maps => "purpose.maps",
            ContainerPurpose::Monitoring => "purpose.monitoring",
            ContainerPurpose::Database => "purpose.database",
            ContainerPurpose::Cache => "purpose.cache",
            ContainerPurpose::Custom => "purpose.custom",
        }
    }

    /// `true` when this service is infrastructure (not directly user-facing).
    pub fn is_infrastructure(self) -> bool {
        matches!(
            self,
            ContainerPurpose::Proxy
                | ContainerPurpose::Database
                | ContainerPurpose::Cache
                | ContainerPurpose::Monitoring
        )
    }
}

// ── TypeEntry ─────────────────────────────────────────────────────────────────

/// A single entry in the type registry, describing one known service type.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeEntry {
    /// Machine identifier, e.g. `"mail/stalwart"`.
    pub id: String,
    /// Broad category, e.g. `"mail"`, `"proxy"`.
    pub category: String,
    /// Human label shown in select fields and lists.
    pub label: String,
    /// Capabilities this type provides when deployed.
    pub provides: Vec<String>,
}

// ── TypeRegistry ─────────────────────────────────────────────────────────────

/// Runtime registry of known service types, loaded from the store catalog.
///
/// Used by TUI `SelectInputNode` for service class selection, by the health
/// checker to validate service classes, and by the capability matcher.
#[derive(Debug, Default)]
pub struct TypeRegistry {
    entries: Vec<TypeEntry>,
}

impl TypeRegistry {
    /// Create an empty registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a new type entry.
    pub fn register(&mut self, entry: TypeEntry) {
        self.entries.push(entry);
    }

    /// All entries in registration order.
    pub fn entries(&self) -> &[TypeEntry] {
        &self.entries
    }

    /// Find an entry by exact id (e.g. `"mail/stalwart"`).
    pub fn get(&self, id: &str) -> Option<&TypeEntry> {
        self.entries.iter().find(|e| e.id == id)
    }

    /// All entries matching a category prefix (e.g. `"mail"`).
    pub fn by_category(&self, category: &str) -> Vec<&TypeEntry> {
        self.entries
            .iter()
            .filter(|e| e.category == category)
            .collect()
    }

    /// All registered category names, deduplicated in insertion order.
    pub fn categories(&self) -> Vec<&str> {
        let mut seen = std::collections::HashSet::new();
        self.entries
            .iter()
            .filter_map(|e| {
                if seen.insert(e.category.as_str()) {
                    Some(e.category.as_str())
                } else {
                    None
                }
            })
            .collect()
    }

    /// Total number of registered entries.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// `true` when the registry contains no entries.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(id: &str, cat: &str) -> TypeEntry {
        TypeEntry {
            id: id.into(),
            category: cat.into(),
            label: id.into(),
            provides: vec![cat.into()],
        }
    }

    #[test]
    fn service_type_category() {
        let t = ServiceType::new("mail/stalwart");
        assert_eq!(t.category(), "mail");
        assert_eq!(t.implementation(), Some("stalwart"));
        assert!(t.is_concrete());
    }

    #[test]
    fn service_type_bare_category() {
        let t = ServiceType::from("proxy");
        assert_eq!(t.category(), "proxy");
        assert_eq!(t.implementation(), None);
        assert!(!t.is_concrete());
    }

    #[test]
    fn container_purpose_is_infrastructure() {
        assert!(ContainerPurpose::Database.is_infrastructure());
        assert!(ContainerPurpose::Proxy.is_infrastructure());
        assert!(!ContainerPurpose::Mail.is_infrastructure());
        assert!(!ContainerPurpose::Wiki.is_infrastructure());
    }

    #[test]
    fn type_registry_by_category() {
        let mut r = TypeRegistry::new();
        r.register(entry("mail/stalwart", "mail"));
        r.register(entry("proxy/zentinel", "proxy"));
        r.register(entry("mail/maddy", "mail"));
        let mail = r.by_category("mail");
        assert_eq!(mail.len(), 2);
    }

    #[test]
    fn type_registry_categories_deduped() {
        let mut r = TypeRegistry::new();
        r.register(entry("mail/stalwart", "mail"));
        r.register(entry("mail/maddy", "mail"));
        r.register(entry("proxy/zentinel", "proxy"));
        let cats = r.categories();
        assert_eq!(cats.len(), 2);
    }

    #[test]
    fn type_registry_get() {
        let mut r = TypeRegistry::new();
        r.register(entry("iam/kanidm", "iam"));
        assert!(r.get("iam/kanidm").is_some());
        assert!(r.get("iam/keycloak").is_none());
    }
}
