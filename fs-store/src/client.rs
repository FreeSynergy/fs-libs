// StoreClient — walks the 3-level FreeSynergy Store catalog tree.
//
// Level 1 (root):      fetch_root()                → RootCatalog
// Level 2 (namespace): fetch_namespace(ns_catalog) → NamespaceCatalog
// Level 3 (package):   fetch_package(pkg_ref)      → PackageCatalog
//
// Bundles/Themes live at root level (bundles/, themes/).
// Their namespace path is resolved by the NamespaceEntry.catalog field
// in the RootCatalog — no special-casing needed in fetch logic.
//
// FTL files:   fetch_ftl(base_path, locale)        → String
//
// Design: Open/Closed Principle via StoreSource.
//   New source kinds (IPFS, OCI registry) = new variant, no existing code touched.

use std::path::Path;

use anyhow::{Context, Result};
use serde::de::DeserializeOwned;
use tracing::{debug, info};

use crate::catalog::{NamespaceCatalog, NamespaceEntry, PackageCatalog, PackageRef, RootCatalog};

// ── StoreSource ───────────────────────────────────────────────────────────────

/// Identifies where a store catalog can be found.
#[derive(Debug, Clone)]
pub enum StoreSource {
    /// Local directory — path is the Store root (dev / CI mode).
    Local(std::path::PathBuf),
    /// HTTP base URL — all catalog paths are appended to this.
    Http(String),
}

impl StoreSource {
    /// Build the full location string for a given relative path within the Store.
    ///
    /// `rel_path` is relative to the Store root, e.g. `"catalog.toml"` or
    /// `"packages/apps/kanidm/catalog.toml"`.
    pub fn resolve(&self, rel_path: &str) -> String {
        match self {
            Self::Local(root) => root.join(rel_path).to_string_lossy().into_owned(),
            Self::Http(base) => format!(
                "{}/{}",
                base.trim_end_matches('/'),
                rel_path.trim_start_matches('/')
            ),
        }
    }
}

// ── StoreClient ───────────────────────────────────────────────────────────────

/// Store catalog client — navigates the 3-level catalog tree.
pub struct StoreClient {
    source: StoreSource,
    http: reqwest::Client,
}

impl StoreClient {
    // ── Constructors ──────────────────────────────────────────────────────────

    /// Create a client for the given source.
    pub fn new(source: StoreSource) -> Self {
        Self {
            source,
            http: reqwest::Client::new(),
        }
    }

    /// Client pointed at the official FreeSynergy Store.
    ///
    /// Override via the `FS_STORE_URL` environment variable.
    pub fn official() -> Self {
        let url = std::env::var("FS_STORE_URL").unwrap_or_else(|_| {
            "https://raw.githubusercontent.com/FreeSynergy/Store/main".to_string()
        });
        info!("StoreClient: official store at {url}");
        Self::new(StoreSource::Http(url))
    }

    // ── Level 1: Root ─────────────────────────────────────────────────────────

    /// Fetch the root `catalog.toml` — the namespace index.
    ///
    /// Call this first to discover all available namespaces, then iterate
    /// over `RootCatalog::namespaces` and call `fetch_namespace` for each.
    pub async fn fetch_root(&mut self) -> Result<RootCatalog> {
        self.fetch_toml("catalog.toml").await
    }

    // ── Level 2: Namespace ────────────────────────────────────────────────────

    /// Fetch a namespace catalog using the path from a `NamespaceEntry`.
    ///
    /// Works for both regular namespaces (`packages/apps/catalog.toml`) and
    /// root-level namespaces (`bundles/catalog.toml`, `themes/catalog.toml`).
    pub async fn fetch_namespace(&mut self, entry: &NamespaceEntry) -> Result<NamespaceCatalog> {
        self.fetch_toml(&entry.catalog).await
    }

    /// Fetch a namespace catalog by path directly.
    pub async fn fetch_namespace_at(&mut self, catalog_path: &str) -> Result<NamespaceCatalog> {
        self.fetch_toml(catalog_path).await
    }

    // ── Level 3: Package ──────────────────────────────────────────────────────

    /// Fetch a package catalog given a `PackageRef` from a namespace catalog.
    ///
    /// `namespace_dir` is the directory of the namespace catalog, used to
    /// resolve the relative `PackageRef::catalog` path.
    ///
    /// Example:
    /// ```text
    /// namespace_dir = "packages/apps"
    /// pkg_ref.catalog = "kanidm/catalog.toml"
    /// → fetches "packages/apps/kanidm/catalog.toml"
    /// ```
    pub async fn fetch_package(
        &mut self,
        namespace_dir: &str,
        pkg_ref: &PackageRef,
    ) -> Result<PackageCatalog> {
        let path = join_catalog_path(namespace_dir, &pkg_ref.catalog);
        self.fetch_toml(&path).await
    }

    /// Fetch a package catalog by exact Store-root-relative path.
    pub async fn fetch_package_at(&mut self, catalog_path: &str) -> Result<PackageCatalog> {
        self.fetch_toml(catalog_path).await
    }

    // ── FTL descriptions ──────────────────────────────────────────────────────

    /// Fetch the long-form `.ftl` description for a package.
    ///
    /// `catalog_dir` is the directory of the package's `catalog.toml`.
    /// `locale` is a BCP 47 locale code, e.g. `"en"` or `"de"`.
    ///
    /// Falls back to `"en"` if the requested locale file is not found.
    ///
    /// Example:
    /// ```text
    /// catalog_dir = "packages/apps/kanidm"
    /// locale      = "de"
    /// → fetches "packages/apps/kanidm/help/de/description.ftl"
    ///   (fallback: "packages/apps/kanidm/help/en/description.ftl")
    /// ```
    pub async fn fetch_ftl(&mut self, catalog_dir: &str, locale: &str) -> Result<String> {
        let primary = format!("{catalog_dir}/help/{locale}/description.ftl");
        match self.fetch_text(&primary).await {
            Ok(text) => Ok(text),
            Err(_) if locale != "en" => {
                debug!("StoreClient: FTL locale '{locale}' not found, falling back to 'en'");
                let fallback = format!("{catalog_dir}/help/en/description.ftl");
                self.fetch_text(&fallback)
                    .await
                    .with_context(|| format!("FTL fallback 'en' not found for {catalog_dir}"))
            }
            Err(e) => Err(e),
        }
    }

    // ── Bundle / Theme helper ─────────────────────────────────────────────────

    /// Resolve the component packages of a Bundle or Theme.
    ///
    /// For each `BundleRef` in `package.bundle.components`, fetches the
    /// referenced package's `PackageCatalog` so the Store UI can display
    /// component details and link to them.
    ///
    /// Components without an explicit `catalog` path are looked up by id
    /// across all namespaces in `root`.
    pub async fn fetch_bundle_components(
        &mut self,
        package: &PackageCatalog,
        root: &RootCatalog,
    ) -> Result<Vec<PackageCatalog>> {
        let refs = match &package.bundle {
            Some(b) => &b.components,
            None => return Ok(vec![]),
        };

        let mut result = Vec::with_capacity(refs.len());
        for component_ref in refs {
            let catalog_path = if let Some(path) = &component_ref.catalog {
                path.clone()
            } else {
                // Resolve by id: search all namespaces for this package id
                resolve_by_id(&component_ref.id, root).with_context(|| {
                    format!(
                        "bundle component '{}' not found in any namespace",
                        component_ref.id
                    )
                })?
            };
            let pkg = self.fetch_package_at(&catalog_path).await?;
            result.push(pkg);
        }
        Ok(result)
    }

    // ── Internal ──────────────────────────────────────────────────────────────

    async fn fetch_toml<T: DeserializeOwned>(&mut self, rel_path: &str) -> Result<T> {
        let text = self.fetch_text(rel_path).await?;
        toml::from_str(&text).with_context(|| format!("parsing TOML from '{rel_path}'"))
    }

    async fn fetch_text(&self, rel_path: &str) -> Result<String> {
        match &self.source {
            StoreSource::Local(root) => {
                let path = root.join(rel_path);
                debug!("StoreClient: reading local {}", path.display());
                std::fs::read_to_string(&path)
                    .with_context(|| format!("reading '{}'", path.display()))
            }
            StoreSource::Http(base) => {
                let url = format!(
                    "{}/{}",
                    base.trim_end_matches('/'),
                    rel_path.trim_start_matches('/')
                );
                debug!("StoreClient: fetching {url}");
                self.http
                    .get(&url)
                    .send()
                    .await
                    .with_context(|| format!("GET {url}"))?
                    .error_for_status()
                    .with_context(|| format!("HTTP error for {url}"))?
                    .text()
                    .await
                    .with_context(|| format!("reading response from {url}"))
            }
        }
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Join a namespace directory and a package-relative catalog path.
///
/// e.g. `join_catalog_path("packages/apps", "kanidm/catalog.toml")`
///      → `"packages/apps/kanidm/catalog.toml"`
fn join_catalog_path(namespace_dir: &str, pkg_catalog: &str) -> String {
    let ns = namespace_dir.trim_end_matches('/');
    let pkg = pkg_catalog.trim_start_matches('/');
    format!("{ns}/{pkg}")
}

/// Resolve a package id to its catalog path by searching the root catalog.
///
/// Builds the canonical path `{ns_dir}/{id}/catalog.toml` for each namespace
/// and returns the first match. This is a best-effort lookup used when
/// a `BundleRef` has no explicit `catalog` field.
fn resolve_by_id(id: &str, root: &RootCatalog) -> Option<String> {
    for ns in &root.namespaces {
        // Derive the namespace directory from its catalog path
        // e.g. "packages/apps/catalog.toml" → "packages/apps"
        let ns_dir = Path::new(&ns.catalog)
            .parent()?
            .to_string_lossy()
            .into_owned();
        let candidate = format!("{ns_dir}/{id}/catalog.toml");
        // We cannot do I/O here — return the path and let the caller verify
        if ns_dir.contains(id) || candidate.contains(&format!("/{id}/")) {
            return Some(candidate);
        }
    }
    None
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn store_source_local_resolve() {
        let src = StoreSource::Local("/home/kal/Server/Store".into());
        assert_eq!(
            src.resolve("catalog.toml"),
            "/home/kal/Server/Store/catalog.toml"
        );
        assert_eq!(
            src.resolve("packages/apps/kanidm/catalog.toml"),
            "/home/kal/Server/Store/packages/apps/kanidm/catalog.toml"
        );
        assert_eq!(
            src.resolve("themes/midnight-blue/catalog.toml"),
            "/home/kal/Server/Store/themes/midnight-blue/catalog.toml"
        );
    }

    #[test]
    fn store_source_http_resolve() {
        let src =
            StoreSource::Http("https://raw.githubusercontent.com/FreeSynergy/Store/main".into());
        assert_eq!(
            src.resolve("catalog.toml"),
            "https://raw.githubusercontent.com/FreeSynergy/Store/main/catalog.toml"
        );
        assert_eq!(
            src.resolve("bundles/zentinel/catalog.toml"),
            "https://raw.githubusercontent.com/FreeSynergy/Store/main/bundles/zentinel/catalog.toml"
        );
    }

    #[test]
    fn store_source_http_trims_trailing_slash() {
        let src =
            StoreSource::Http("https://raw.githubusercontent.com/FreeSynergy/Store/main/".into());
        let url = src.resolve("catalog.toml");
        assert!(!url.contains("//catalog"), "double slash in URL: {url}");
    }

    #[test]
    fn join_catalog_path_combines_correctly() {
        assert_eq!(
            join_catalog_path("packages/apps", "kanidm/catalog.toml"),
            "packages/apps/kanidm/catalog.toml"
        );
        assert_eq!(
            join_catalog_path("themes/", "midnight-blue/catalog.toml"),
            "themes/midnight-blue/catalog.toml"
        );
    }

    #[tokio::test]
    async fn fetch_root_parses_local_store() {
        let src = StoreSource::Local("/home/kal/Server/Store".into());
        let mut client = StoreClient::new(src);
        let root = client
            .fetch_root()
            .await
            .expect("root catalog should parse");
        assert_eq!(root.catalog.id, "freesynergy-store");
        assert!(!root.namespaces.is_empty());
        // bundles and themes must be at root level (not under packages/)
        let bundles_ns = root.namespaces.iter().find(|n| n.id == "bundles");
        let themes_ns = root.namespaces.iter().find(|n| n.id == "themes");
        assert!(
            bundles_ns.is_some(),
            "bundles namespace missing from root catalog"
        );
        assert!(
            themes_ns.is_some(),
            "themes namespace missing from root catalog"
        );
        assert!(
            bundles_ns.unwrap().catalog.starts_with("bundles/"),
            "bundles catalog path should start with 'bundles/', got: {}",
            bundles_ns.unwrap().catalog
        );
        assert!(
            themes_ns.unwrap().catalog.starts_with("themes/"),
            "themes catalog path should start with 'themes/', got: {}",
            themes_ns.unwrap().catalog
        );
    }

    #[tokio::test]
    async fn fetch_namespace_apps_local() {
        let src = StoreSource::Local("/home/kal/Server/Store".into());
        let mut client = StoreClient::new(src);
        let ns_entry = crate::catalog::NamespaceEntry {
            id: "apps".into(),
            name: "Applications".into(),
            r#type: Some("app".into()),
            catalog: "packages/apps/catalog.toml".into(),
        };
        let ns = client
            .fetch_namespace(&ns_entry)
            .await
            .expect("apps namespace catalog");
        assert!(!ns.is_empty());
        assert!(ns.packages.iter().any(|p| p.id == "kanidm"));
    }

    #[tokio::test]
    async fn fetch_package_kanidm_local() {
        let src = StoreSource::Local("/home/kal/Server/Store".into());
        let mut client = StoreClient::new(src);
        let pkg_ref = crate::catalog::PackageRef {
            id: "kanidm".into(),
            catalog: "kanidm/catalog.toml".into(),
        };
        let pkg = client
            .fetch_package("packages/apps", &pkg_ref)
            .await
            .expect("kanidm package catalog");
        assert_eq!(pkg.id(), "kanidm");
        assert!(!pkg.is_bundle());
    }

    #[tokio::test]
    async fn fetch_bundle_zentinel_local() {
        let src = StoreSource::Local("/home/kal/Server/Store".into());
        let mut client = StoreClient::new(src);
        let pkg_ref = crate::catalog::PackageRef {
            id: "zentinel".into(),
            catalog: "zentinel/catalog.toml".into(),
        };
        let bundle = client
            .fetch_package("bundles", &pkg_ref)
            .await
            .expect("zentinel bundle catalog");
        assert_eq!(bundle.id(), "zentinel");
        assert!(bundle.is_bundle(), "zentinel should be a bundle");
    }

    #[tokio::test]
    async fn fetch_ftl_en_local() {
        let src = StoreSource::Local("/home/kal/Server/Store".into());
        let mut client = StoreClient::new(src);
        let ftl = client
            .fetch_ftl("packages/apps/kanidm", "en")
            .await
            .expect("kanidm en FTL");
        assert!(ftl.contains("kanidm"), "FTL should mention kanidm");
    }

    #[tokio::test]
    async fn fetch_ftl_falls_back_to_en() {
        let src = StoreSource::Local("/home/kal/Server/Store".into());
        let mut client = StoreClient::new(src);
        // "fr" locale does not exist — should fall back to "en"
        let ftl = client
            .fetch_ftl("packages/apps/kanidm", "fr")
            .await
            .expect("kanidm fr→en FTL fallback");
        assert!(
            ftl.contains("kanidm"),
            "fallback FTL should still contain kanidm"
        );
    }
}
