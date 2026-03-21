//! Module-related types: runtime status and installation source.
//!
//! A "module" in FreeSynergy is a containerized service definition — e.g. `mail/stalwart`,
//! `iam/kanidm`. It is fetched from the store, installed via Quadlet, and tracked here.

use serde::{Deserialize, Serialize};

// ── ModuleStatus ──────────────────────────────────────────────────────────────

/// Runtime status of an installed module (containerized service).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ModuleStatus {
    /// Container is running and healthy.
    Running,
    /// Container is stopped (manually or by policy).
    Stopped,
    /// Module is being installed or started for the first time.
    Installing,
    /// Module is being updated to a new version.
    Updating,
    /// Container exited with an error; restart may be in progress.
    Error,
    /// Module is registered but not yet scheduled on a host.
    #[default]
    Pending,
}

impl ModuleStatus {
    /// Human-readable label for UI display.
    pub fn label(self) -> &'static str {
        match self {
            ModuleStatus::Running    => "Running",
            ModuleStatus::Stopped    => "Stopped",
            ModuleStatus::Installing => "Installing",
            ModuleStatus::Updating   => "Updating",
            ModuleStatus::Error      => "Error",
            ModuleStatus::Pending    => "Pending",
        }
    }

    /// i18n key.
    pub fn i18n_key(self) -> &'static str {
        match self {
            ModuleStatus::Running    => "module.status.running",
            ModuleStatus::Stopped    => "module.status.stopped",
            ModuleStatus::Installing => "module.status.installing",
            ModuleStatus::Updating   => "module.status.updating",
            ModuleStatus::Error      => "module.status.error",
            ModuleStatus::Pending    => "module.status.pending",
        }
    }

    /// `true` when the module is actively serving traffic.
    pub fn is_running(self) -> bool {
        matches!(self, ModuleStatus::Running)
    }

    /// `true` when the module is in a transitional state.
    pub fn is_transitioning(self) -> bool {
        matches!(self, ModuleStatus::Installing | ModuleStatus::Updating)
    }

    /// `true` when the module needs operator attention.
    pub fn needs_attention(self) -> bool {
        matches!(self, ModuleStatus::Error)
    }
}

// ── ModuleSource ──────────────────────────────────────────────────────────────

/// Where a module definition was obtained from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ModuleSource {
    /// Downloaded from the official FreeSynergy store.
    #[default]
    Store,
    /// Loaded from a local path on disk.
    Local,
    /// Provided by a third-party external source URL.
    External,
}

impl ModuleSource {
    /// Human-readable label for UI display.
    pub fn label(self) -> &'static str {
        match self {
            ModuleSource::Store    => "Store",
            ModuleSource::Local    => "Local",
            ModuleSource::External => "External",
        }
    }

    /// i18n key.
    pub fn i18n_key(self) -> &'static str {
        match self {
            ModuleSource::Store    => "module.source.store",
            ModuleSource::Local    => "module.source.local",
            ModuleSource::External => "module.source.external",
        }
    }

    /// `true` when the module can be auto-updated from its source.
    pub fn supports_auto_update(self) -> bool {
        matches!(self, ModuleSource::Store | ModuleSource::External)
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn running_module_is_running() {
        assert!(ModuleStatus::Running.is_running());
        assert!(!ModuleStatus::Stopped.is_running());
    }

    #[test]
    fn installing_is_transitioning() {
        assert!(ModuleStatus::Installing.is_transitioning());
        assert!(ModuleStatus::Updating.is_transitioning());
        assert!(!ModuleStatus::Running.is_transitioning());
    }

    #[test]
    fn error_needs_attention() {
        assert!(ModuleStatus::Error.needs_attention());
        assert!(!ModuleStatus::Running.needs_attention());
    }

    #[test]
    fn store_source_supports_auto_update() {
        assert!(ModuleSource::Store.supports_auto_update());
        assert!(ModuleSource::External.supports_auto_update());
        assert!(!ModuleSource::Local.supports_auto_update());
    }

    #[test]
    fn module_status_default_is_pending() {
        assert_eq!(ModuleStatus::default(), ModuleStatus::Pending);
    }

    #[test]
    fn module_source_default_is_store() {
        assert_eq!(ModuleSource::default(), ModuleSource::Store);
    }
}
