// fsn-store — Universal store client for FreeSynergy module registry.
//
// Migrated and enhanced from Store/crates/store-sdk.
// Uses fsn-error instead of anyhow, adds catalog caching and retry logic.
//
// Design:
//   StoreSource    — Strategy: where data comes from (local path / HTTP)
//   StoreClient    — fetches catalog + i18n from a StoreSource
//   Manifest trait — project-supplied package type (FSN module, Wiki plugin, …)
//   Catalog<M>     — generic catalog container with filtering
//   CatalogCache   — in-memory TTL cache for fetched catalogs
//
// Pattern: Strategy (StoreSource), Generic Container (Catalog<M>)

pub mod catalog;
pub mod client;
pub mod disk_cache;
pub mod i18n;
pub mod manifest;
pub mod permissions;
pub mod search;

pub use catalog::{Catalog, CatalogCache, CatalogMeta, LocaleEntry};
pub use client::{RetryPolicy, StoreClient, StoreSource};
pub use disk_cache::DiskCache;
pub use i18n::{I18nBundle, I18nMeta, TextDirection};
pub use manifest::{Manifest, PackageCompat, PackageMeta, PackageSource};
pub use permissions::{StoreAction, StorePermissions, StoreRole};
pub use search::{HasTags, SearchQuery, SearchResult, StoreSearch};
