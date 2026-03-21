// client.rs — StoreSource + StoreClient.
//
// StoreSource is a Strategy that abstracts where data comes from:
//   - Local(path) → reads files from disk (offline / development)
//   - Http(url)   → fetches via reqwest with retry + offline fallback
//
// StoreClient wraps a StoreSource and uses:
//   - CatalogCache  — in-memory TTL cache (avoids redundant fetches)
//   - DiskCache     — on-disk fallback (used when HTTP fails = offline mode)
//   - backon        — exponential backoff retry on transient HTTP errors
//
// Retry policy (HTTP only):
//   - 3 attempts, exponential backoff: 1s → 2s → 4s
//   - On final failure: fall back to disk cache if available
//   - If disk cache also empty: return the original network error
//
// Fetch paths:
//   catalog.toml  → {base}/{namespace}/catalog.toml
//   ui.toml       → {base}/{namespace}/i18n/{locale}/ui.toml
//   i18n manifest → {base}/{namespace}/i18n/{locale}/manifest.toml

use std::path::PathBuf;
use std::time::Duration;

use backon::{ExponentialBuilder, Retryable};
use fs_error::FsError;
use serde::Deserialize;
use tracing::{debug, warn};

use crate::catalog::{Catalog, CatalogCache};
use crate::disk_cache::DiskCache;
use crate::i18n::{I18nBundle, I18nMeta};
use crate::manifest::Manifest;

// ── RetryPolicy ───────────────────────────────────────────────────────────────

/// Retry configuration for HTTP fetches.
#[derive(Debug, Clone)]
pub struct RetryPolicy {
    /// Maximum number of retry attempts (not counting the first try).
    pub max_retries: u32,
    /// Initial backoff delay.
    pub initial_delay: Duration,
}

impl RetryPolicy {
    /// Default: 3 retries, 1s initial backoff (exponential: 1s → 2s → 4s).
    pub fn default_backoff() -> Self {
        Self { max_retries: 3, initial_delay: Duration::from_secs(1) }
    }

    fn builder(&self) -> ExponentialBuilder {
        ExponentialBuilder::default()
            .with_max_times(self.max_retries as usize)
            .with_min_delay(self.initial_delay)
    }
}

// ── StoreSource ───────────────────────────────────────────────────────────────

/// Where the store data comes from — local filesystem or HTTP.
///
/// Implements the Strategy pattern: `StoreClient` delegates all I/O here.
#[derive(Debug, Clone)]
pub enum StoreSource {
    /// Read files from a local directory (offline / development).
    Local(PathBuf),
    /// Fetch files over HTTP (production).
    Http(String),
}

impl StoreSource {
    /// Create an HTTP source.
    pub fn http(base_url: impl Into<String>) -> Self {
        Self::Http(base_url.into())
    }

    /// Create a local filesystem source.
    pub fn local(path: impl Into<PathBuf>) -> Self {
        Self::Local(path.into())
    }

    /// The base URL or path as a string (for cache keys and logging).
    pub fn base(&self) -> String {
        match self {
            Self::Local(p) => p.display().to_string(),
            Self::Http(url) => url.trim_end_matches('/').to_string(),
        }
    }

    /// Fetch raw text from `{base}/{relative_path}` — no retry, no cache.
    ///
    /// For HTTP sources with retry + offline fallback, use
    /// [`StoreSource::fetch_text_with_retry`].
    async fn fetch_text_once(&self, relative_path: &str) -> Result<String, FsError> {
        match self {
            Self::Local(root) => {
                let path = root.join(relative_path);
                debug!("store read local: {}", path.display());
                tokio::fs::read_to_string(&path).await.map_err(|e| {
                    FsError::internal(format!("store local read '{}': {e}", path.display()))
                })
            }
            Self::Http(base) => {
                let url = format!("{}/{}", base.trim_end_matches('/'), relative_path);
                debug!("store fetch: {url}");
                let client = reqwest::Client::builder()
                    .timeout(Duration::from_secs(15))
                    .build()
                    .map_err(|e| FsError::internal(format!("reqwest build: {e}")))?;
                let resp = client.get(&url).send().await.map_err(|e| {
                    FsError::network(format!("store http fetch '{url}': {e}"))
                })?;
                if !resp.status().is_success() {
                    return Err(FsError::network(format!(
                        "store http {}: {}",
                        resp.status(),
                        url
                    )));
                }
                resp.text().await.map_err(|e| {
                    FsError::network(format!("store http read body '{url}': {e}"))
                })
            }
        }
    }

    /// Fetch with retry (HTTP only) and offline fallback via `disk_cache`.
    ///
    /// 1. Try `fetch_text_once` up to `policy.max_retries + 1` times.
    /// 2. If all attempts fail: check `disk_cache` for a stale hit.
    /// 3. If disk also empty: return the last network error.
    ///
    /// For `Local` sources, calls `fetch_text_once` directly (no retry needed).
    async fn fetch_text_with_retry(
        &self,
        relative_path: &str,
        policy: &RetryPolicy,
        disk_cache: &DiskCache,
    ) -> Result<String, FsError> {
        // Local sources: no retry, no disk cache
        if matches!(self, Self::Local(_)) {
            return self.fetch_text_once(relative_path).await;
        }

        let cache_key = format!("{}/{}", self.base(), relative_path);

        // Retry with exponential backoff
        let result = {
            let source = self.clone();
            let path = relative_path.to_owned();
            let backoff = policy.builder();
            (move || {
                let source = source.clone();
                let path = path.clone();
                async move { source.fetch_text_once(&path).await }
            })
            .retry(backoff)
            .await
        };

        match result {
            Ok(text) => {
                // Persist successful fetch to disk for future offline use
                disk_cache.insert(&cache_key, &text);
                Ok(text)
            }
            Err(network_err) => {
                // All retries failed — try offline disk cache
                if let Some(stale) = disk_cache.get(&cache_key) {
                    warn!(
                        path = relative_path,
                        "store: network failed, serving stale disk cache"
                    );
                    Ok(stale)
                } else {
                    Err(network_err)
                }
            }
        }
    }
}

// ── StoreClient ───────────────────────────────────────────────────────────────

/// Client for reading catalogs and i18n bundles from a `StoreSource`.
///
/// Uses `CatalogCache` to avoid redundant fetches within a session.
/// For HTTP sources a 5-minute TTL is applied; local sources are re-read
/// on every call (no caching) so development edits are picked up immediately.
///
/// HTTP fetches use exponential backoff retry (default: 3 retries) and fall
/// back to an on-disk cache when all retries fail (offline mode).
pub struct StoreClient {
    source: StoreSource,
    cache: CatalogCache,
    disk_cache: DiskCache,
    retry: RetryPolicy,
}

impl StoreClient {
    /// Create a new client with a 5-minute catalog cache and default retry policy.
    pub fn new(source: StoreSource) -> Self {
        Self {
            cache: match &source {
                StoreSource::Http(_) => CatalogCache::default_ttl(),
                StoreSource::Local(_) => CatalogCache::new(Duration::ZERO),
            },
            disk_cache: DiskCache::default_location(),
            retry: RetryPolicy::default_backoff(),
            source,
        }
    }

    /// Override the retry policy.
    pub fn with_retry(mut self, policy: RetryPolicy) -> Self {
        self.retry = policy;
        self
    }

    /// Override the disk cache location.
    pub fn with_disk_cache(mut self, cache: DiskCache) -> Self {
        self.disk_cache = cache;
        self
    }

    /// Create a production client pointing at the FreeSynergy Store.
    ///
    /// URL: `https://raw.githubusercontent.com/FreeSynergy/Store/main`
    pub fn node_store() -> Self {
        Self::new(StoreSource::http(
            "https://raw.githubusercontent.com/FreeSynergy/Store/main",
        ))
    }

    /// Fetch and parse the catalog for `namespace`, e.g. `"node"`.
    ///
    /// Result is cached; pass `force = true` to bypass the in-memory cache.
    pub async fn fetch_catalog<M>(&mut self, namespace: &str, force: bool) -> Result<Catalog<M>, FsError>
    where
        M: Manifest + for<'de> Deserialize<'de>,
    {
        let relative = format!("{namespace}/catalog.toml");
        let cache_key = format!("{}/{}", self.source.base(), relative);

        if !force {
            if let Some(cached) = self.cache.get(&cache_key) {
                debug!("store cache hit: {cache_key}");
                return Catalog::from_toml(cached);
            }
        }

        let text = self.source
            .fetch_text_with_retry(&relative, &self.retry, &self.disk_cache)
            .await?;
        self.cache.insert(cache_key, text.clone());
        Catalog::from_toml(&text)
    }

    /// Fetch an i18n bundle for `namespace` + `locale_code` (e.g. `"de"`).
    ///
    /// Loads `{namespace}/i18n/{locale}/manifest.toml` + `ui.toml`.
    pub async fn fetch_i18n(&self, namespace: &str, locale_code: &str) -> Result<I18nBundle, FsError> {
        let meta_path = format!("{namespace}/i18n/{locale_code}/manifest.toml");
        let ui_path   = format!("{namespace}/i18n/{locale_code}/ui.toml");

        let meta_text = self.source
            .fetch_text_with_retry(&meta_path, &self.retry, &self.disk_cache)
            .await?;
        let ui_text = self.source
            .fetch_text_with_retry(&ui_path, &self.retry, &self.disk_cache)
            .await?;

        let meta: I18nManifestWrapper = toml::from_str(&meta_text)
            .map_err(|e| FsError::parse(format!("i18n manifest parse: {e}")))?;

        let ui: toml::Value = toml::from_str(&ui_text)
            .map_err(|e| FsError::parse(format!("ui.toml parse: {e}")))?;

        Ok(I18nBundle { meta: meta.i18n, ui })
    }

    /// Fetch raw text from any path relative to the store base.
    ///
    /// Uses retry + offline fallback for HTTP sources.
    pub async fn fetch_raw(&self, relative_path: &str) -> Result<String, FsError> {
        self.source
            .fetch_text_with_retry(relative_path, &self.retry, &self.disk_cache)
            .await
    }

    /// Invalidate the in-memory cached catalog for `namespace`.
    pub fn invalidate(&mut self, namespace: &str) {
        let relative = format!("{namespace}/catalog.toml");
        let key = format!("{}/{}", self.source.base(), relative);
        warn!("store cache invalidated: {key}");
        self.cache.invalidate(&key);
    }

    /// Evict all expired entries from the in-memory catalog cache.
    pub fn evict_expired(&mut self) {
        self.cache.evict_expired();
    }

    /// The store source this client reads from.
    pub fn source(&self) -> &StoreSource {
        &self.source
    }
}

// ── private helpers ───────────────────────────────────────────────────────────

/// Wrapper for the `[i18n]` block in a locale `manifest.toml`.
#[derive(Deserialize)]
struct I18nManifestWrapper {
    i18n: I18nMeta,
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manifest::{Manifest, PackageMeta};
    use serde::Deserialize;
    use std::io::Write;
    use tempfile::TempDir;

    /// Minimal local package type for testing.
    #[derive(Debug, Clone, Deserialize)]
    struct TestPkg {
        #[serde(flatten)]
        meta: PackageMeta,
    }

    impl Manifest for TestPkg {
        fn id(&self) -> &str       { &self.meta.id }
        fn version(&self) -> &str  { &self.meta.version }
        fn category(&self) -> &str { &self.meta.category }
        fn name(&self) -> &str     { &self.meta.name }
    }

    fn write(dir: &std::path::Path, rel: &str, content: &str) {
        let path = dir.join(rel);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        let mut f = std::fs::File::create(&path).unwrap();
        f.write_all(content.as_bytes()).unwrap();
    }

    #[tokio::test]
    async fn local_fetch_catalog() {
        let dir = TempDir::new().unwrap();

        write(
            dir.path(),
            "TestNS/catalog.toml",
            r#"
[catalog]
project     = "TestNS"
version     = "1.0.0"
generated_at = "2026-01-01"

[[packages]]
id          = "pkg-a"
name        = "Package A"
version     = "0.1.0"
category    = "test.a"
description = "desc"
license     = "MIT"
author      = "tester"
"#,
        );

        let mut client = StoreClient::new(StoreSource::local(dir.path()));
        let catalog: Catalog<TestPkg> = client.fetch_catalog("TestNS", false).await.unwrap();

        assert_eq!(catalog.packages.len(), 1);
        assert_eq!(catalog.packages[0].id(), "pkg-a");
    }

    #[tokio::test]
    async fn local_fetch_i18n() {
        let dir = TempDir::new().unwrap();

        write(
            dir.path(),
            "NS/i18n/de/manifest.toml",
            r#"
[i18n]
locale_code  = "de"
native_name  = "Deutsch"
completeness = 100
"#,
        );
        write(
            dir.path(),
            "NS/i18n/de/ui.toml",
            r#"
[welcome]
title = "Willkommen"
"#,
        );

        let client = StoreClient::new(StoreSource::local(dir.path()));
        let bundle = client.fetch_i18n("NS", "de").await.unwrap();

        assert_eq!(bundle.meta.locale_code, "de");
        let map = bundle.to_hashmap();
        assert_eq!(map.get("welcome.title").map(String::as_str), Some("Willkommen"));
    }

    #[tokio::test]
    async fn offline_fallback_uses_disk_cache() {
        let _dir = TempDir::new().unwrap();
        let disk_dir = TempDir::new().unwrap();

        // Pre-seed disk cache with stale catalog
        let disk_cache = DiskCache::new(disk_dir.path());
        disk_cache.insert(
            "https://example.invalid/store/NS/catalog.toml",
            r#"
[catalog]
project     = "NS"
version     = "0.1.0"
generated_at = "2026-01-01"

[[packages]]
id          = "stale-pkg"
name        = "Stale Package"
version     = "0.1.0"
category    = "test"
description = "from disk cache"
license     = "MIT"
author      = "tester"
"#,
        );

        // HTTP source pointing at an unreachable URL
        let source = StoreSource::http("https://example.invalid/store");
        let policy = RetryPolicy { max_retries: 0, initial_delay: Duration::from_millis(1) };

        let result = source
            .fetch_text_with_retry("NS/catalog.toml", &policy, &disk_cache)
            .await;

        assert!(result.is_ok(), "should fall back to disk cache");
        let text = result.unwrap();
        assert!(text.contains("stale-pkg"));
    }
}
