// search.rs — Full-text search over store catalogs.
//
// Searches packages by query string across one or more namespaces.
// Ranking is simple: exact ID match > name match > tag match > description match.
//
// Design:
//   SearchQuery     — what to search for and where
//   SearchResult<M> — one matching package with its score and namespace
//   StoreSearch<M>  — executes queries against one or more Catalog<M>
//
// Pattern: Value object (SearchQuery), Ranker (score_package).

use crate::manifest::Manifest;

// ── SearchQuery ───────────────────────────────────────────────────────────────

/// A search request.
#[derive(Debug, Clone)]
pub struct SearchQuery {
    /// The search string (case-insensitive substring match).
    pub query: String,

    /// If set, only return packages in this category prefix (e.g. `"deploy.iam"`).
    pub category_filter: Option<String>,

    /// If set, only return packages from this namespace (e.g. `"node"`).
    pub namespace_filter: Option<String>,

    /// Maximum number of results to return (0 = unlimited).
    pub limit: usize,
}

impl SearchQuery {
    /// Create a simple query with no filters.
    pub fn new(query: impl Into<String>) -> Self {
        Self {
            query:            query.into(),
            category_filter:  None,
            namespace_filter: None,
            limit:            0,
        }
    }

    /// Set the category filter.
    pub fn with_category(mut self, category: impl Into<String>) -> Self {
        self.category_filter = Some(category.into());
        self
    }

    /// Set the namespace filter.
    pub fn in_namespace(mut self, namespace: impl Into<String>) -> Self {
        self.namespace_filter = Some(namespace.into());
        self
    }

    /// Set the result limit.
    pub fn limit(mut self, n: usize) -> Self {
        self.limit = n;
        self
    }
}

// ── SearchResult ──────────────────────────────────────────────────────────────

/// One search result.
#[derive(Debug, Clone)]
pub struct SearchResult<M> {
    /// The matching package.
    pub package: M,

    /// The store namespace this package belongs to.
    pub namespace: String,

    /// Relevance score (higher = better match).
    pub score: u32,
}

// ── Scoring ───────────────────────────────────────────────────────────────────

/// Score weights.
const SCORE_EXACT_ID:    u32 = 100;
const SCORE_EXACT_NAME:  u32 = 80;
const SCORE_ID_CONTAINS: u32 = 60;
const SCORE_NAME_SUBSTR: u32 = 40;
const SCORE_TAG_MATCH:   u32 = 20;
const SCORE_CATEGORY:    u32 = 10;

/// Compute a relevance score for a package against a lowercase query string.
///
/// The caller is responsible for lowercasing the query.
pub fn score_package<M: Manifest + HasTags>(pkg: &M, query_lower: &str) -> u32 {
    if query_lower.is_empty() {
        return 1; // everything matches an empty query
    }

    let id_lower   = pkg.id().to_lowercase();
    let name_lower = pkg.name().to_lowercase();
    let cat_lower  = pkg.category().to_lowercase();

    if id_lower == query_lower   { return SCORE_EXACT_ID; }
    if name_lower == query_lower { return SCORE_EXACT_NAME; }
    if id_lower.contains(query_lower)   { return SCORE_ID_CONTAINS; }
    if name_lower.contains(query_lower) { return SCORE_NAME_SUBSTR; }

    if pkg.tags().iter().any(|t| t.to_lowercase().contains(query_lower)) {
        return SCORE_TAG_MATCH;
    }

    if cat_lower.contains(query_lower) {
        return SCORE_CATEGORY;
    }

    0
}

// ── HasTags trait ─────────────────────────────────────────────────────────────

/// Extension trait for packages that expose a tag list.
///
/// Implemented by any manifest type that has tags. Search falls back to
/// zero tag score for types that don't implement this.
pub trait HasTags {
    fn tags(&self) -> &[String];
}

// ── StoreSearch ───────────────────────────────────────────────────────────────

/// Executes searches across one or more namespaced catalogs.
///
/// # Example
///
/// ```rust
/// use fsn_store::search::{StoreSearch, SearchQuery, HasTags};
/// use fsn_store::catalog::Catalog;
/// use fsn_store::manifest::{Manifest, PackageMeta};
///
/// #[derive(Clone, serde::Deserialize)]
/// struct MyPkg { package: PackageMeta }
///
/// impl Manifest for MyPkg {
///     fn id(&self)       -> &str { &self.package.id }
///     fn version(&self)  -> &str { &self.package.version }
///     fn category(&self) -> &str { &self.package.category }
///     fn name(&self)     -> &str { &self.package.name }
/// }
///
/// impl HasTags for MyPkg {
///     fn tags(&self) -> &[String] { &self.package.tags }
/// }
///
/// let mut search: StoreSearch<MyPkg> = StoreSearch::new();
/// // (add catalogs, then call search.query(...))
/// ```
pub struct StoreSearch<M> {
    /// Pairs of (namespace, packages).
    catalogs: Vec<(String, Vec<M>)>,
}

impl<M: Manifest + HasTags + Clone> StoreSearch<M> {
    /// Create an empty search engine.
    pub fn new() -> Self {
        Self { catalogs: Vec::new() }
    }

    /// Add a namespace catalog to the search index.
    pub fn add_namespace(&mut self, namespace: impl Into<String>, packages: Vec<M>) {
        let ns = namespace.into();
        // Replace existing namespace if already present.
        self.catalogs.retain(|(n, _)| n != &ns);
        self.catalogs.push((ns, packages));
    }

    /// Remove a namespace from the search index.
    pub fn remove_namespace(&mut self, namespace: &str) {
        self.catalogs.retain(|(n, _)| n != namespace);
    }

    /// Execute a search query and return sorted results (highest score first).
    pub fn query(&self, q: &SearchQuery) -> Vec<SearchResult<M>> {
        let query_lower = q.query.to_lowercase();

        let mut results: Vec<SearchResult<M>> = self
            .catalogs
            .iter()
            .filter(|(ns, _)| {
                q.namespace_filter
                    .as_ref()
                    .map(|f| ns.to_lowercase() == f.to_lowercase())
                    .unwrap_or(true)
            })
            .flat_map(|(ns, pkgs)| {
                pkgs.iter()
                    .filter(|p| {
                        q.category_filter
                            .as_ref()
                            .map(|f| p.category().starts_with(f.as_str()))
                            .unwrap_or(true)
                    })
                    .filter_map(|p| {
                        let s = score_package(p, &query_lower);
                        if s > 0 {
                            Some(SearchResult {
                                package:   p.clone(),
                                namespace: ns.clone(),
                                score:     s,
                            })
                        } else {
                            None
                        }
                    })
            })
            .collect();

        // Sort by score descending, then by ID ascending for determinism.
        results.sort_by(|a, b| {
            b.score
                .cmp(&a.score)
                .then_with(|| a.package.id().cmp(b.package.id()))
        });

        if q.limit > 0 {
            results.truncate(q.limit);
        }

        results
    }

    /// Search with a plain string (no filters, no limit).
    pub fn find(&self, query: &str) -> Vec<SearchResult<M>> {
        self.query(&SearchQuery::new(query))
    }
}

impl<M: Manifest + HasTags + Clone> Default for StoreSearch<M> {
    fn default() -> Self {
        Self::new()
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manifest::{Manifest, PackageMeta};
    use serde::Deserialize;

    #[derive(Debug, Clone, Deserialize)]
    struct TestPkg {
        package: PackageMeta,
    }

    impl Manifest for TestPkg {
        fn id(&self)       -> &str { &self.package.id }
        fn version(&self)  -> &str { &self.package.version }
        fn category(&self) -> &str { &self.package.category }
        fn name(&self)     -> &str { &self.package.name }
    }

    impl HasTags for TestPkg {
        fn tags(&self) -> &[String] { &self.package.tags }
    }

    fn make_pkg(id: &str, name: &str, category: &str, tags: &[&str]) -> TestPkg {
        TestPkg {
            package: PackageMeta {
                id:          id.into(),
                name:        name.into(),
                version:     "1.0.0".into(),
                category:    category.into(),
                description: format!("{} description", name),
                license:     "MIT".into(),
                author:      "FSN".into(),
                tags:        tags.iter().map(|s| s.to_string()).collect(),
                source:      None,
                compat:      None,
            },
        }
    }

    fn test_search() -> StoreSearch<TestPkg> {
        let mut s = StoreSearch::new();
        s.add_namespace("node", vec![
            make_pkg("iam/kanidm",       "Kanidm",       "deploy.iam",      &["oidc", "scim", "mfa"]),
            make_pkg("iam/keycloak",     "KeyCloak",     "deploy.iam",      &["oidc", "saml"]),
            make_pkg("proxy/zentinel",   "Zentinel",     "deploy.proxy",    &["proxy", "tls"]),
            make_pkg("wiki/outline",     "Outline",      "deploy.wiki",     &["wiki", "docs"]),
            make_pkg("git/forgejo",      "Forgejo",      "deploy.git",      &["git", "forge"]),
            make_pkg("mail/stalwart",    "Stalwart",     "deploy.mail",     &["smtp", "imap"]),
        ]);
        s.add_namespace("shared", vec![
            make_pkg("theme/dark",       "Dark Theme",   "themes",          &["dark", "theme"]),
        ]);
        s
    }

    #[test]
    fn exact_id_match_scores_highest() {
        let s = test_search();
        let results = s.find("iam/kanidm");
        assert!(!results.is_empty());
        assert_eq!(results[0].package.id(), "iam/kanidm");
        assert_eq!(results[0].score, SCORE_EXACT_ID);
    }

    #[test]
    fn exact_name_match() {
        let s = test_search();
        let results = s.find("kanidm");
        assert!(!results.is_empty());
        // "iam/kanidm" ID contains "kanidm" → SCORE_ID_CONTAINS
        assert!(results[0].score >= SCORE_ID_CONTAINS);
    }

    #[test]
    fn tag_match() {
        let s = test_search();
        let results = s.find("oidc");
        let ids: Vec<&str> = results.iter().map(|r| r.package.id()).collect();
        assert!(ids.contains(&"iam/kanidm"));
        assert!(ids.contains(&"iam/keycloak"));
    }

    #[test]
    fn category_filter() {
        let s = test_search();
        let q = SearchQuery::new("").with_category("deploy.iam");
        let results = s.query(&q);
        assert_eq!(results.len(), 2);
        let ids: Vec<&str> = results.iter().map(|r| r.package.id()).collect();
        assert!(ids.contains(&"iam/kanidm"));
        assert!(ids.contains(&"iam/keycloak"));
    }

    #[test]
    fn namespace_filter() {
        let s = test_search();
        let q = SearchQuery::new("").in_namespace("shared");
        let results = s.query(&q);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].namespace, "shared");
    }

    #[test]
    fn limit() {
        let s = test_search();
        let q = SearchQuery::new("").limit(2);
        let results = s.query(&q);
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn empty_query_returns_all() {
        let s = test_search();
        let results = s.find("");
        assert_eq!(results.len(), 7); // all packages
    }

    #[test]
    fn no_results_for_unknown_term() {
        let s = test_search();
        let results = s.find("zzz-nonexistent");
        assert!(results.is_empty());
    }

    #[test]
    fn sorted_by_score_desc() {
        let s = test_search();
        let results = s.find("iam");
        // All results should be in descending score order.
        for window in results.windows(2) {
            assert!(window[0].score >= window[1].score);
        }
    }

    #[test]
    fn remove_namespace() {
        let mut s = test_search();
        s.remove_namespace("shared");
        let q = SearchQuery::new("").in_namespace("shared");
        assert!(s.query(&q).is_empty());
    }
}
