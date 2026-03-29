//! Project-related types: lifecycle status and visibility.

use serde::{Deserialize, Serialize};

// ── ProjectStatus ─────────────────────────────────────────────────────────────

/// Lifecycle status of a `FreeSynergy` deployment project.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ProjectStatus {
    /// Project is actively deployed and maintained.
    #[default]
    Active,
    /// Project has been put into long-term storage (read-only).
    Archived,
    /// Project is defined but not yet deployed to any host.
    Pending,
    /// Project encountered an unrecoverable error during deployment.
    Error,
}

impl ProjectStatus {
    /// Human-readable label for UI display.
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            ProjectStatus::Active => "Active",
            ProjectStatus::Archived => "Archived",
            ProjectStatus::Pending => "Pending",
            ProjectStatus::Error => "Error",
        }
    }

    /// i18n key.
    #[must_use]
    pub fn i18n_key(self) -> &'static str {
        match self {
            ProjectStatus::Active => "project.status.active",
            ProjectStatus::Archived => "project.status.archived",
            ProjectStatus::Pending => "project.status.pending",
            ProjectStatus::Error => "project.status.error",
        }
    }

    /// `true` when new services can be deployed into this project.
    #[must_use]
    pub fn allows_deployment(self) -> bool {
        matches!(self, ProjectStatus::Active | ProjectStatus::Pending)
    }

    /// `true` when the project is in a terminal/read-only state.
    #[must_use]
    pub fn is_read_only(self) -> bool {
        matches!(self, ProjectStatus::Archived)
    }
}

// ── ProjectVisibility ─────────────────────────────────────────────────────────

/// Who can see and interact with a project in the UI / store.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ProjectVisibility {
    /// Only the owner and explicitly added members can see this project.
    #[default]
    Private,
    /// All authenticated users on this node can see the project.
    Internal,
    /// Visible to guests and unauthenticated users (store listing).
    Public,
}

impl ProjectVisibility {
    /// Human-readable label for UI display.
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            ProjectVisibility::Private => "Private",
            ProjectVisibility::Internal => "Internal",
            ProjectVisibility::Public => "Public",
        }
    }

    /// i18n key.
    #[must_use]
    pub fn i18n_key(self) -> &'static str {
        match self {
            ProjectVisibility::Private => "project.visibility.private",
            ProjectVisibility::Internal => "project.visibility.internal",
            ProjectVisibility::Public => "project.visibility.public",
        }
    }

    /// `true` when unauthenticated users can read this project.
    #[must_use]
    pub fn is_public(self) -> bool {
        matches!(self, ProjectVisibility::Public)
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn active_project_allows_deployment() {
        assert!(ProjectStatus::Active.allows_deployment());
    }

    #[test]
    fn archived_project_is_read_only() {
        assert!(ProjectStatus::Archived.is_read_only());
        assert!(!ProjectStatus::Active.is_read_only());
    }

    #[test]
    fn error_project_does_not_allow_deployment() {
        assert!(!ProjectStatus::Error.allows_deployment());
    }

    #[test]
    fn private_visibility_is_not_public() {
        assert!(!ProjectVisibility::Private.is_public());
        assert!(ProjectVisibility::Public.is_public());
    }

    #[test]
    fn project_status_default_is_active() {
        assert_eq!(ProjectStatus::default(), ProjectStatus::Active);
    }

    #[test]
    fn project_visibility_default_is_private() {
        assert_eq!(ProjectVisibility::default(), ProjectVisibility::Private);
    }
}
