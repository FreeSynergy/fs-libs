// permissions.rs — Role-based access control for FreeSynergy Store operations.
//
// Store permissions follow a simple four-tier model:
//   Admin      — everything on all nodes
//   NodeAdmin  — install/remove on own nodes only
//   User       — themes, languages, widgets on own desktop
//   Guest      — read-only / view only
//
// Design:
//   StoreRole      — the four roles
//   StoreAction    — actions that can be performed in the store
//   StorePermissions — checks whether a role may perform an action
//
// Pattern: Enum with methods (StoreRole carries its own permission logic).

use serde::{Deserialize, Serialize};

// ── StoreRole ─────────────────────────────────────────────────────────────────

/// A user's role in the FreeSynergy Store.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum StoreRole {
    /// View only — cannot install anything.
    Guest = 0,
    /// Can change themes, languages, and widgets on their own desktop.
    User = 1,
    /// Can install/remove services on nodes they administer.
    NodeAdmin = 2,
    /// Full control — can install/remove on all nodes.
    Admin = 3,
}

impl StoreRole {
    /// Human-readable label.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Guest     => "Guest",
            Self::User      => "User",
            Self::NodeAdmin => "Node Admin",
            Self::Admin     => "Admin",
        }
    }

    /// Short description of what this role can do.
    pub fn description(&self) -> &'static str {
        match self {
            Self::Guest     => "View the store; cannot install anything",
            Self::User      => "Change themes, languages, and widgets on own desktop",
            Self::NodeAdmin => "Install and remove services on own nodes",
            Self::Admin     => "Full control on all nodes",
        }
    }

    /// Returns `true` if this role can perform the given action.
    pub fn can(&self, action: StoreAction) -> bool {
        StorePermissions::check(*self, action)
    }
}

// ── StoreAction ───────────────────────────────────────────────────────────────

/// An action that can be performed in the store.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StoreAction {
    /// Browse and view package listings.
    ViewCatalog,

    /// Download a package manifest or archive.
    DownloadPackage,

    /// Install a service package on any node (Admin only).
    InstallServiceAnyNode,

    /// Install a service package on an administered node.
    InstallServiceOwnNode,

    /// Remove a service package from any node (Admin only).
    RemoveServiceAnyNode,

    /// Remove a service package from an administered node.
    RemoveServiceOwnNode,

    /// Apply a theme, language pack, or widget on own desktop.
    ApplyDesktopCustomization,

    /// Publish a new package to the store (Admin only).
    PublishPackage,

    /// Delete a package from the store (Admin only).
    DeletePackage,

    /// Manage store users and roles (Admin only).
    ManageUsers,
}

// ── StorePermissions ──────────────────────────────────────────────────────────

/// Stateless permission checker for store operations.
///
/// # Example
///
/// ```rust
/// use fs_store::permissions::{StoreRole, StoreAction, StorePermissions};
///
/// assert!(StorePermissions::check(StoreRole::Admin, StoreAction::PublishPackage));
/// assert!(!StorePermissions::check(StoreRole::Guest, StoreAction::InstallServiceOwnNode));
/// assert!(StorePermissions::check(StoreRole::User, StoreAction::ApplyDesktopCustomization));
/// ```
pub struct StorePermissions;

impl StorePermissions {
    /// Returns `true` if `role` may perform `action`.
    pub fn check(role: StoreRole, action: StoreAction) -> bool {
        match action {
            // Everyone can view and browse.
            StoreAction::ViewCatalog
            | StoreAction::DownloadPackage => true,

            // Desktop customisation: User and above.
            StoreAction::ApplyDesktopCustomization => role >= StoreRole::User,

            // Installing/removing on own nodes: NodeAdmin and above.
            StoreAction::InstallServiceOwnNode
            | StoreAction::RemoveServiceOwnNode => role >= StoreRole::NodeAdmin,

            // Everything else: Admin only.
            StoreAction::InstallServiceAnyNode
            | StoreAction::RemoveServiceAnyNode
            | StoreAction::PublishPackage
            | StoreAction::DeletePackage
            | StoreAction::ManageUsers => role == StoreRole::Admin,
        }
    }

    /// Returns all actions that `role` is permitted to perform.
    pub fn allowed_actions(role: StoreRole) -> Vec<StoreAction> {
        let all = [
            StoreAction::ViewCatalog,
            StoreAction::DownloadPackage,
            StoreAction::ApplyDesktopCustomization,
            StoreAction::InstallServiceOwnNode,
            StoreAction::RemoveServiceOwnNode,
            StoreAction::InstallServiceAnyNode,
            StoreAction::RemoveServiceAnyNode,
            StoreAction::PublishPackage,
            StoreAction::DeletePackage,
            StoreAction::ManageUsers,
        ];
        all.iter().filter(|&&a| Self::check(role, a)).copied().collect()
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn guest_can_only_view() {
        assert!(StoreRole::Guest.can(StoreAction::ViewCatalog));
        assert!(StoreRole::Guest.can(StoreAction::DownloadPackage));
        assert!(!StoreRole::Guest.can(StoreAction::ApplyDesktopCustomization));
        assert!(!StoreRole::Guest.can(StoreAction::InstallServiceOwnNode));
        assert!(!StoreRole::Guest.can(StoreAction::PublishPackage));
    }

    #[test]
    fn user_can_customize_desktop() {
        assert!(StoreRole::User.can(StoreAction::ViewCatalog));
        assert!(StoreRole::User.can(StoreAction::ApplyDesktopCustomization));
        assert!(!StoreRole::User.can(StoreAction::InstallServiceOwnNode));
        assert!(!StoreRole::User.can(StoreAction::ManageUsers));
    }

    #[test]
    fn node_admin_can_manage_own_nodes() {
        assert!(StoreRole::NodeAdmin.can(StoreAction::InstallServiceOwnNode));
        assert!(StoreRole::NodeAdmin.can(StoreAction::RemoveServiceOwnNode));
        assert!(!StoreRole::NodeAdmin.can(StoreAction::InstallServiceAnyNode));
        assert!(!StoreRole::NodeAdmin.can(StoreAction::PublishPackage));
        assert!(!StoreRole::NodeAdmin.can(StoreAction::ManageUsers));
    }

    #[test]
    fn admin_can_do_everything() {
        let all_actions = [
            StoreAction::ViewCatalog,
            StoreAction::DownloadPackage,
            StoreAction::ApplyDesktopCustomization,
            StoreAction::InstallServiceOwnNode,
            StoreAction::RemoveServiceOwnNode,
            StoreAction::InstallServiceAnyNode,
            StoreAction::RemoveServiceAnyNode,
            StoreAction::PublishPackage,
            StoreAction::DeletePackage,
            StoreAction::ManageUsers,
        ];
        for action in &all_actions {
            assert!(StoreRole::Admin.can(*action), "Admin should be able to {action:?}");
        }
    }

    #[test]
    fn allowed_actions_count() {
        let guest_actions = StorePermissions::allowed_actions(StoreRole::Guest);
        let user_actions  = StorePermissions::allowed_actions(StoreRole::User);
        let na_actions    = StorePermissions::allowed_actions(StoreRole::NodeAdmin);
        let admin_actions = StorePermissions::allowed_actions(StoreRole::Admin);

        assert!(guest_actions.len() < user_actions.len());
        assert!(user_actions.len()  < na_actions.len());
        assert!(na_actions.len()    < admin_actions.len());
    }

    #[test]
    fn role_ordering() {
        assert!(StoreRole::Admin > StoreRole::NodeAdmin);
        assert!(StoreRole::NodeAdmin > StoreRole::User);
        assert!(StoreRole::User > StoreRole::Guest);
    }

    #[test]
    fn role_labels() {
        assert_eq!(StoreRole::Admin.label(), "Admin");
        assert_eq!(StoreRole::NodeAdmin.label(), "Node Admin");
        assert_eq!(StoreRole::User.label(), "User");
        assert_eq!(StoreRole::Guest.label(), "Guest");
    }

    #[test]
    fn serde_roundtrip() {
        let role = StoreRole::NodeAdmin;
        let json = serde_json::to_string(&role).unwrap();
        let back: StoreRole = serde_json::from_str(&json).unwrap();
        assert_eq!(back, role);
    }
}
