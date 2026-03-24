//! `BundleResource` — a meta-package referencing other resources.

use super::meta::ResourceMeta;
use serde::{Deserialize, Serialize};

// ── BundleEntry ───────────────────────────────────────────────────────────────

/// A single resource referenced inside a bundle.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BundleEntry {
    /// The resource id, e.g. `"midnight-blue-colors"`.
    pub id: String,
    /// When `true`, installation continues even if this entry is unavailable.
    pub optional: bool,
}

impl BundleEntry {
    pub fn required(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            optional: false,
        }
    }

    pub fn optional(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            optional: true,
        }
    }
}

// ── ThemeBundleRefs ───────────────────────────────────────────────────────────

/// Convenience struct for theme bundles — at most one resource per theme slot.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ThemeBundleRefs {
    pub color_scheme: Option<String>,
    pub style: Option<String>,
    pub font_set: Option<String>,
    pub cursor_set: Option<String>,
    pub icon_set: Option<String>,
    pub button_style: Option<String>,
    pub window_chrome: Option<String>,
    pub animation_set: Option<String>,
}

// ── BundleResource ────────────────────────────────────────────────────────────

/// A meta-package that groups other resources and installs them together.
///
/// All referenced resource ids **must exist in the store** for the bundle
/// to pass validation (`ValidationStatus::Ok`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BundleResource {
    /// Shared metadata present on every resource.
    pub meta: ResourceMeta,
    /// Generic list of resource references (for non-theme bundles).
    pub packages: Vec<BundleEntry>,
    /// Convenience theme-slot mapping (for theme bundles; mutually exclusive
    /// with a non-empty `packages` list).
    pub theme: Option<ThemeBundleRefs>,
}
