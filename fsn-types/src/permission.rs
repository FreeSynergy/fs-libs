//! Permission types: actions and scopes for the RBAC system.

use serde::{Deserialize, Serialize};

// ── Action ────────────────────────────────────────────────────────────────────

/// An operation that can be permitted or denied by the RBAC system.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Action {
    /// Read configuration, status, or list resources.
    Read,
    /// Write or modify configuration.
    Write,
    /// Install a new module or resource.
    Install,
    /// Remove an installed module or resource.
    Remove,
    /// Start a stopped service.
    Start,
    /// Stop a running service.
    Stop,
    /// Change runtime configuration (env vars, network, etc.).
    Configure,
    /// Full administrative access — implies all other actions.
    Admin,
}

impl Action {
    /// Human-readable label for UI display.
    pub fn label(self) -> &'static str {
        match self {
            Action::Read      => "Read",
            Action::Write     => "Write",
            Action::Install   => "Install",
            Action::Remove    => "Remove",
            Action::Start     => "Start",
            Action::Stop      => "Stop",
            Action::Configure => "Configure",
            Action::Admin     => "Admin",
        }
    }

    /// i18n key.
    pub fn i18n_key(self) -> &'static str {
        match self {
            Action::Read      => "permission.action.read",
            Action::Write     => "permission.action.write",
            Action::Install   => "permission.action.install",
            Action::Remove    => "permission.action.remove",
            Action::Start     => "permission.action.start",
            Action::Stop      => "permission.action.stop",
            Action::Configure => "permission.action.configure",
            Action::Admin     => "permission.action.admin",
        }
    }

    /// `true` when this action implies all others (super-admin).
    pub fn is_admin(self) -> bool {
        matches!(self, Action::Admin)
    }

    /// All non-admin actions, in declaration order.
    pub fn non_admin_variants() -> &'static [Action] {
        &[
            Action::Read,
            Action::Write,
            Action::Install,
            Action::Remove,
            Action::Start,
            Action::Stop,
            Action::Configure,
        ]
    }
}

// ── Scope ─────────────────────────────────────────────────────────────────────

/// The resource scope to which a permission applies.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "type", content = "id")]
pub enum Scope {
    /// Applies to all resources on all nodes (superuser).
    Global,
    /// Applies to all resources on a specific node (by node slug).
    Node(String),
    /// Applies to all resources within a specific project.
    Project(String),
    /// Applies to a single service (by service slug).
    Service(String),
    /// Applies to a single plugin (by plugin slug).
    Plugin(String),
}

impl Scope {
    /// `true` when this scope covers the entire system.
    pub fn is_global(&self) -> bool {
        matches!(self, Scope::Global)
    }

    /// Return the contained resource ID, if any.
    pub fn resource_id(&self) -> Option<&str> {
        match self {
            Scope::Global           => None,
            Scope::Node(id)
            | Scope::Project(id)
            | Scope::Service(id)
            | Scope::Plugin(id)     => Some(id.as_str()),
        }
    }

    /// Human-readable label for UI display.
    pub fn label(&self) -> String {
        match self {
            Scope::Global        => "Global".to_owned(),
            Scope::Node(id)      => format!("Node:{id}"),
            Scope::Project(id)   => format!("Project:{id}"),
            Scope::Service(id)   => format!("Service:{id}"),
            Scope::Plugin(id)    => format!("Plugin:{id}"),
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn admin_action_is_admin() {
        assert!(Action::Admin.is_admin());
        assert!(!Action::Read.is_admin());
    }

    #[test]
    fn non_admin_variants_excludes_admin() {
        let variants = Action::non_admin_variants();
        assert!(!variants.contains(&Action::Admin));
        assert!(variants.contains(&Action::Read));
    }

    #[test]
    fn global_scope_is_global() {
        assert!(Scope::Global.is_global());
        assert!(!Scope::Node("n1".into()).is_global());
    }

    #[test]
    fn scope_resource_id() {
        assert_eq!(Scope::Global.resource_id(), None);
        assert_eq!(Scope::Project("proj1".into()).resource_id(), Some("proj1"));
        assert_eq!(Scope::Service("kanidm".into()).resource_id(), Some("kanidm"));
    }

    #[test]
    fn scope_label_global() {
        assert_eq!(Scope::Global.label(), "Global");
    }

    #[test]
    fn scope_label_node() {
        assert_eq!(Scope::Node("vps-01".into()).label(), "Node:vps-01");
    }
}
