// manifest.rs — Manifest trait and PackageMeta types.
//
// Every project-specific package type implements the Manifest trait so the
// generic catalog infrastructure can work with any namespace.

use serde::{Deserialize, Serialize};

/// Common interface for all package types across all FreeSynergy namespaces.
pub trait Manifest {
    fn id(&self) -> &str;
    fn version(&self) -> &str;
    fn category(&self) -> &str;
    fn name(&self) -> &str;
}

/// The `[package]` block in every `manifest.toml`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageMeta {
    pub id:          String,
    pub name:        String,
    pub version:     String,
    pub category:    String,
    pub description: String,
    pub license:     String,
    pub author:      String,
    #[serde(default)]
    pub tags:        Vec<String>,
    #[serde(default)]
    pub source:      Option<PackageSource>,
    #[serde(default)]
    pub compat:      Option<PackageCompat>,
}

/// `[package.source]` — upstream project links.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageSource {
    pub website:    Option<String>,
    pub repository: Option<String>,
}

/// `[package.compat]` — version and namespace constraints.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageCompat {
    pub min_app_version: Option<String>,
    #[serde(default)]
    pub projects: Vec<String>,
}
