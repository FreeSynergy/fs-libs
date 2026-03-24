// Catalog types for the 3-level FreeSynergy Store structure.
//
// Level 1 — Root:      catalog.toml           → RootCatalog
// Level 2 — Namespace: {ns}/catalog.toml      → NamespaceCatalog
// Level 3 — Package:   {ns}/{pkg}/catalog.toml → PackageCatalog
//
// Bundles and Themes live at root level (bundles/, themes/) rather than
// under packages/. Their PackageCatalog uses BundleRef to list components,
// each of which links back to an individual package catalog.

use serde::{Deserialize, Serialize};

// ── Level 1: Root catalog ─────────────────────────────────────────────────────

/// The root `catalog.toml` — the namespace index the StoreClient reads first.
///
/// Lists all namespaces (packages/apps, packages/containers, bundles, themes, …)
/// with their catalog paths and dominant resource type.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RootCatalog {
    /// Root catalog header.
    #[serde(default)]
    pub catalog: RootCatalogMeta,

    /// All namespaces in this store.
    #[serde(default, alias = "namespace")]
    pub namespaces: Vec<NamespaceEntry>,
}

/// Header of the root catalog.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RootCatalogMeta {
    /// Unique store identifier, e.g. `"freesynergy-store"`.
    #[serde(default)]
    pub id: String,
    /// Human-readable store name.
    #[serde(default)]
    pub name: String,
    /// Path to the store's SVG icon.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    /// One-line summary of the store.
    #[serde(default)]
    pub summary: String,
    /// Catalog format version.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
}

/// One namespace entry in the root catalog.
///
/// Points to either a `packages/{ns}/catalog.toml` (regular packages)
/// or a root-level `{ns}/catalog.toml` (bundles, themes, init).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NamespaceEntry {
    /// Namespace slug, e.g. `"apps"`, `"bundles"`, `"themes"`.
    pub id: String,
    /// Human-readable namespace name.
    pub name: String,
    /// Dominant resource type for this namespace, e.g. `"app"`, `"theme"`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,
    /// Relative path from Store root to the namespace `catalog.toml`.
    pub catalog: String,
}

// ── Level 2: Namespace catalog ────────────────────────────────────────────────

/// A namespace-level `catalog.toml` — lists all packages in one namespace.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NamespaceCatalog {
    /// Namespace catalog header (id, name, icon, summary).
    #[serde(default)]
    pub catalog: NamespaceCatalogMeta,

    /// Package references — each points to an individual `catalog.toml`.
    #[serde(default, alias = "package")]
    pub packages: Vec<PackageRef>,
}

impl NamespaceCatalog {
    /// Returns `true` if the namespace contains no packages.
    pub fn is_empty(&self) -> bool {
        self.packages.is_empty()
    }

    /// Number of packages in this namespace.
    pub fn len(&self) -> usize {
        self.packages.len()
    }
}

/// Header of a namespace catalog.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NamespaceCatalogMeta {
    /// Namespace slug.
    #[serde(default)]
    pub id: String,
    /// Human-readable namespace name.
    #[serde(default)]
    pub name: String,
    /// Path to the namespace's SVG icon.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    /// One-line summary.
    #[serde(default)]
    pub summary: String,
}

/// A pointer from a namespace catalog to one package's individual catalog.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageRef {
    /// Package slug, e.g. `"kanidm"`, `"midnight-blue"`.
    pub id: String,
    /// Relative path from the namespace directory to the package `catalog.toml`.
    /// e.g. `"kanidm/catalog.toml"`.
    pub catalog: String,
}

// ── Level 3: Package catalog ──────────────────────────────────────────────────

/// A fully parsed individual package `catalog.toml`.
///
/// Contains all metadata: identity, descriptions, type, source, provides,
/// requires, variables, contract routes, and — for bundles/themes — a list
/// of component references.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PackageCatalog {
    /// Package metadata block.
    #[serde(default)]
    pub package: PackageMeta,

    /// Component references — present only for Bundle and Theme packages.
    /// Each entry links to an individual package by id so the Store UI
    /// can display the component's full detail page via a link.
    #[serde(default)]
    pub bundle: Option<BundleBlock>,
}

impl PackageCatalog {
    /// Convenience: the package id.
    pub fn id(&self) -> &str {
        &self.package.id
    }

    /// Convenience: the package name.
    pub fn name(&self) -> &str {
        &self.package.name
    }

    /// `true` when this package is a Bundle or Theme (has a `[bundle]` block).
    pub fn is_bundle(&self) -> bool {
        self.bundle.is_some()
    }

    /// Returns the component ids if this is a Bundle/Theme, or an empty slice.
    pub fn component_ids(&self) -> Vec<&str> {
        self.bundle
            .as_ref()
            .map(|b| b.components.iter().map(|c| c.id.as_str()).collect())
            .unwrap_or_default()
    }
}

/// The `[package]` block inside a package `catalog.toml`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PackageMeta {
    /// Unique slug, e.g. `"kanidm"`.
    #[serde(default)]
    pub id: String,
    /// Human-readable name.
    #[serde(default)]
    pub name: String,
    /// Resource type, e.g. `"app"`, `"container"`, `"theme"`, `"bundle"`.
    #[serde(default)]
    pub r#type: String,
    /// SemVer version.
    #[serde(default)]
    pub version: String,
    /// ≤255 char summary for store listings.
    #[serde(default)]
    pub summary: String,
    /// Medium-length description for the detail view (inline in catalog).
    #[serde(default)]
    pub description: String,
    /// Relative path to the long-form `.ftl` description file.
    #[serde(default)]
    pub description_file: String,
    /// Path to the SVG icon.
    #[serde(default)]
    pub icon: String,
    /// Author or organisation.
    #[serde(default)]
    pub author: String,
    /// SPDX license identifier.
    #[serde(default)]
    pub license: String,
    /// Search/filter tags.
    #[serde(default)]
    pub tags: Vec<String>,
}

/// The `[bundle]` block inside a Bundle or Theme `catalog.toml`.
/// Lists the components that make up this bundle.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BundleBlock {
    /// Component references — each points to another package by id.
    #[serde(default)]
    pub components: Vec<BundleRef>,
}

/// A single component reference inside a `[bundle]` block.
///
/// The Store UI renders this as a link to the referenced package's detail page.
/// The Store object resolves the id to the actual `PackageCatalog` on demand.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BundleRef {
    /// The referenced package id, e.g. `"zentinel"`.
    pub id: String,
    /// Optional relative catalog path — if absent, the Store resolves by id
    /// from the known namespaces.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub catalog: Option<String>,
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn app_catalog(id: &str) -> PackageCatalog {
        PackageCatalog {
            package: PackageMeta {
                id: id.into(),
                name: "Test App".into(),
                r#type: "app".into(),
                version: "1.0.0".into(),
                summary: "A test application for the store.".into(),
                description: "Full description of the test application.".into(),
                description_file: "help/en/description.ftl".into(),
                icon: "icon.svg".into(),
                author: "FreeSynergy".into(),
                license: "MIT".into(),
                tags: vec!["test".into()],
            },
            bundle: None,
        }
    }

    fn bundle_catalog(id: &str, component_ids: &[&str]) -> PackageCatalog {
        PackageCatalog {
            package: PackageMeta {
                id: id.into(),
                name: "Test Bundle".into(),
                r#type: "bundle".into(),
                version: "1.0.0".into(),
                summary: "A test bundle for the store.".into(),
                description: "This bundle groups multiple packages.".into(),
                description_file: "help/en/description.ftl".into(),
                icon: "icon.svg".into(),
                author: "FreeSynergy".into(),
                license: "MIT".into(),
                tags: vec!["bundle".into()],
            },
            bundle: Some(BundleBlock {
                components: component_ids
                    .iter()
                    .map(|id| BundleRef {
                        id: id.to_string(),
                        catalog: None,
                    })
                    .collect(),
            }),
        }
    }

    #[test]
    fn package_catalog_app_is_not_bundle() {
        let pkg = app_catalog("kanidm");
        assert!(!pkg.is_bundle());
        assert!(pkg.component_ids().is_empty());
    }

    #[test]
    fn package_catalog_bundle_is_bundle() {
        let pkg = bundle_catalog("zentinel", &["zentinel", "zentinel-control-plane"]);
        assert!(pkg.is_bundle());
        assert_eq!(
            pkg.component_ids(),
            vec!["zentinel", "zentinel-control-plane"]
        );
    }

    #[test]
    fn package_catalog_id_and_name() {
        let pkg = app_catalog("kanidm");
        assert_eq!(pkg.id(), "kanidm");
        assert_eq!(pkg.name(), "Test App");
    }

    #[test]
    fn namespace_catalog_len_and_empty() {
        let mut ns = NamespaceCatalog::default();
        assert!(ns.is_empty());
        assert_eq!(ns.len(), 0);
        ns.packages.push(PackageRef {
            id: "kanidm".into(),
            catalog: "kanidm/catalog.toml".into(),
        });
        assert!(!ns.is_empty());
        assert_eq!(ns.len(), 1);
    }

    #[test]
    fn root_catalog_parses_from_toml() {
        let toml = r#"
            [catalog]
            id      = "freesynergy-store"
            name    = "FreeSynergy Store"
            summary = "The official store"
            version = "1.0.0"

            [[namespaces]]
            id      = "apps"
            name    = "Applications"
            type    = "app"
            catalog = "packages/apps/catalog.toml"

            [[namespaces]]
            id      = "themes"
            name    = "Themes"
            type    = "theme"
            catalog = "themes/catalog.toml"
        "#;
        let root: RootCatalog = toml::from_str(toml).unwrap();
        assert_eq!(root.catalog.id, "freesynergy-store");
        assert_eq!(root.namespaces.len(), 2);
        assert_eq!(root.namespaces[0].id, "apps");
        assert_eq!(root.namespaces[1].id, "themes");
        assert_eq!(root.namespaces[1].catalog, "themes/catalog.toml");
    }

    #[test]
    fn bundle_ref_without_explicit_catalog() {
        let b = BundleRef {
            id: "zentinel".into(),
            catalog: None,
        };
        assert_eq!(b.id, "zentinel");
        assert!(b.catalog.is_none());
    }
}
