// fs-store — FreeSynergy Store SDK
//
// Generic catalog client for the FreeSynergy package ecosystem.
//
// Public API surface:
//   StoreClient      — walks the 3-level catalog tree (root → namespace → package)
//   StoreSource      — Local(PathBuf) or Http(String)
//   RootCatalog      — root catalog.toml: namespace index
//   NamespaceEntry   — one namespace listed in the root catalog
//   NamespaceCatalog — namespace-level catalog.toml: package-ref list
//   PackageRef       — pointer from a namespace catalog to a package catalog
//   PackageCatalog   — individual package catalog.toml (full metadata)
//   BundleRef        — a component entry inside a Bundle/Theme package catalog
//   Manifest         — trait any catalog entry must implement (Strategy Pattern)

mod catalog;
mod client;
mod manifest;

pub use catalog::{
    BundleRef, NamespaceCatalog, NamespaceEntry, PackageCatalog, PackageRef, RootCatalog,
};
pub use client::{StoreClient, StoreSource};
pub use manifest::Manifest;
