// Manifest trait — Strategy Pattern for store entries.
//
// Every type that can appear as a catalog entry must implement `Manifest`.
// This decouples the StoreClient from concrete entry types.
//
// OOP principle: behavior belongs to the type — callers filter/sort entries
// via the trait interface, not by matching concrete fields.

use crate::catalog::PackageCatalog;

/// A catalog entry that can be fetched, listed, and looked up by id.
pub trait Manifest: Send + Sync {
    /// Unique identifier within the catalog, e.g. `"kanidm"`.
    fn id(&self) -> &str;

    /// SemVer version string, e.g. `"1.3.0"`.
    fn version(&self) -> &str;

    /// Namespace / category this entry belongs to, e.g. `"apps"`, `"themes"`.
    fn namespace(&self) -> &str;

    /// Human-readable display name, e.g. `"Kanidm"`.
    fn name(&self) -> &str;
}

// ── PackageCatalog implements Manifest ────────────────────────────────────────

impl Manifest for PackageCatalog {
    fn id(&self) -> &str {
        &self.package.id
    }

    fn version(&self) -> &str {
        &self.package.version
    }

    fn namespace(&self) -> &str {
        // The type field doubles as a namespace hint when no explicit context exists.
        &self.package.r#type
    }

    fn name(&self) -> &str {
        &self.package.name
    }
}
