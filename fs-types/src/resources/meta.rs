//! Common metadata carried by every store resource.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use super::platform::PlatformFilter;

// ── Role ──────────────────────────────────────────────────────────────────────

/// A standardized role identifier, e.g. `"iam"`, `"wiki"`, `"git"`, `"chat"`.
///
/// Roles are the vocabulary the Bus and Inventory use to route requests.
/// Services declare which roles they fulfil; Bridges map those roles to real APIs.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Role(pub String);

impl Role {
    /// Create from any string-like value.
    pub fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }

    /// The inner role string, e.g. `"iam"`.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for Role {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl From<&str> for Role {
    fn from(s: &str) -> Self {
        Self(s.to_owned())
    }
}

// ── ResourceType ──────────────────────────────────────────────────────────────

/// Discriminant for the 16 resource kinds a FreeSynergy store can contain.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResourceType {
    // ── Programs ──────────────────────────────────────────────────────────────
    /// A native FreeSynergy binary app (Node, Desktop, Conductor, …).
    App,
    /// A containerized application bundled with its Compose YAML (Kanidm, Forgejo, …).
    Container,

    // ── Collections ───────────────────────────────────────────────────────────
    /// A meta-package that references other resources by id.
    Bundle,

    // ── Functional resources ──────────────────────────────────────────────────
    /// A desktop widget with data-source requirements.
    Widget,
    /// A bot that connects to messaging channels and reacts to Bus events.
    Bot,
    /// A bridge mapping a standardized role API to a concrete service API.
    Bridge,
    /// An automation pipeline (Data Offer / Data Accept pair).
    Task,
    /// A language package (Mozilla Fluent snippets).
    Language,

    // ── Theme resources (individually loadable) ───────────────────────────────
    /// A colour palette (CSS custom properties).
    ColorScheme,
    /// Spacing, radius, shadows — strictly standardized schema.
    Style,
    /// Font face files + CSS `@font-face` declarations.
    FontSet,
    /// Mouse cursor files.
    CursorSet,
    /// SVG icon collection.
    IconSet,
    /// Button appearance (shape, border, padding, hover effect).
    ButtonStyle,
    /// Window title-bar style.
    WindowChrome,
    /// CSS transitions and keyframe animations.
    AnimationSet,
    /// A messenger platform adapter — implements the `Channel` trait for one platform.
    MessengerAdapter,
}

impl ResourceType {
    /// Human-readable label for UI display.
    pub fn label(self) -> &'static str {
        match self {
            ResourceType::App          => "App",
            ResourceType::Container => "Container",
            ResourceType::Bundle       => "Bundle",
            ResourceType::Widget       => "Widget",
            ResourceType::Bot          => "Bot",
            ResourceType::Bridge       => "Bridge",
            ResourceType::Task         => "Task",
            ResourceType::Language     => "Language",
            ResourceType::ColorScheme  => "Color Scheme",
            ResourceType::Style        => "Style",
            ResourceType::FontSet      => "Font Set",
            ResourceType::CursorSet    => "Cursor Set",
            ResourceType::IconSet      => "Icon Set",
            ResourceType::ButtonStyle  => "Button Style",
            ResourceType::WindowChrome => "Window Chrome",
            ResourceType::AnimationSet    => "Animation Set",
            ResourceType::MessengerAdapter => "Messenger Adapter",
        }
    }

    /// i18n key for translations.
    pub fn i18n_key(self) -> &'static str {
        match self {
            ResourceType::App          => "resource.type.app",
            ResourceType::Container => "resource.type.container",
            ResourceType::Bundle       => "resource.type.bundle",
            ResourceType::Widget       => "resource.type.widget",
            ResourceType::Bot          => "resource.type.bot",
            ResourceType::Bridge       => "resource.type.bridge",
            ResourceType::Task         => "resource.type.task",
            ResourceType::Language     => "resource.type.language",
            ResourceType::ColorScheme  => "resource.type.color_scheme",
            ResourceType::Style        => "resource.type.style",
            ResourceType::FontSet      => "resource.type.font_set",
            ResourceType::CursorSet    => "resource.type.cursor_set",
            ResourceType::IconSet      => "resource.type.icon_set",
            ResourceType::ButtonStyle  => "resource.type.button_style",
            ResourceType::WindowChrome => "resource.type.window_chrome",
            ResourceType::AnimationSet    => "resource.type.animation_set",
            ResourceType::MessengerAdapter => "resource.type.messenger_adapter",
        }
    }

    /// `true` when this resource is a theme component (individually loadable).
    pub fn is_theme_component(self) -> bool {
        matches!(
            self,
            ResourceType::ColorScheme
                | ResourceType::Style
                | ResourceType::FontSet
                | ResourceType::CursorSet
                | ResourceType::IconSet
                | ResourceType::ButtonStyle
                | ResourceType::WindowChrome
                | ResourceType::AnimationSet
        )
    }
}

// ── ValidationStatus ──────────────────────────────────────────────────────────

/// Continuously-visible integrity status of a store resource.
///
/// Shown as ✅ / ⚠️ / ❌ in every list and detail panel.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ValidationStatus {
    /// All mandatory fields present, signature valid, dependencies resolvable.
    Ok,
    /// Non-critical fields missing (short description, no tags, …).
    #[default]
    Incomplete,
    /// Mandatory fields absent, signature invalid, or dependencies unresolvable.
    Broken,
}

impl ValidationStatus {
    /// Unicode status badge for compact UI display.
    pub fn badge(self) -> &'static str {
        match self {
            ValidationStatus::Ok         => "✅",
            ValidationStatus::Incomplete => "⚠️",
            ValidationStatus::Broken     => "❌",
        }
    }

    /// i18n key.
    pub fn i18n_key(self) -> &'static str {
        match self {
            ValidationStatus::Ok         => "validation.ok",
            ValidationStatus::Incomplete => "validation.incomplete",
            ValidationStatus::Broken     => "validation.broken",
        }
    }

    /// `true` when the resource can be safely installed.
    pub fn is_installable(self) -> bool {
        !matches!(self, ValidationStatus::Broken)
    }
}

// ── Dependency ────────────────────────────────────────────────────────────────

/// A declared dependency on another store resource.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dependency {
    /// The resource id this package depends on, e.g. `"kanidm"`.
    pub id: String,
    /// SemVer requirement string, e.g. `">=1.0.0"` or `"^2"`.
    pub version_req: String,
    /// When `true`, installation continues in degraded mode if unresolvable.
    pub optional: bool,
}

impl Dependency {
    /// Create a mandatory dependency with a version requirement.
    pub fn required(id: impl Into<String>, version_req: impl Into<String>) -> Self {
        Self { id: id.into(), version_req: version_req.into(), optional: false }
    }

    /// Create an optional dependency.
    pub fn optional(id: impl Into<String>, version_req: impl Into<String>) -> Self {
        Self { id: id.into(), version_req: version_req.into(), optional: true }
    }
}

// ── PackageSource ─────────────────────────────────────────────────────────────

/// Upstream provenance of a store resource.
///
/// Tells the Container Manager and Catalog editor where the package comes from
/// so users can submit improvements back to the right place.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageSource {
    /// Container registry, e.g. `"docker.io"`, `"ghcr.io"`, `"quay.io"`.
    pub registry: String,
    /// Full image name without tag, e.g. `"ollama/ollama"`.
    pub image: Option<String>,
    /// Upstream Git repository URL, e.g. `"https://github.com/ollama/ollama"`.
    pub git_repository: Option<String>,
    /// Upstream project website URL.
    pub website: Option<String>,
}

// ── ResourceMeta ──────────────────────────────────────────────────────────────

/// Common mandatory fields carried by every store resource.
///
/// This struct is always embedded as `pub meta: ResourceMeta` in every
/// resource-specific struct (`AppResource`, `WidgetResource`, …).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceMeta {
    /// Unique slug identifier (URL-safe, lowercase, hyphens), e.g. `"kanidm"`.
    pub id: String,
    /// Human-readable display name, e.g. `"Kanidm"`.
    pub name: String,
    /// Short description shown in store listings (≥ 10 characters, mandatory).
    pub description: String,
    /// SemVer version string, e.g. `"1.5.0"`.
    pub version: String,
    /// Author or organisation name.
    pub author: String,
    /// SPDX license identifier, e.g. `"MIT"` or `"Apache-2.0"`.
    pub license: String,
    /// Path to the SVG icon file — mandatory for every resource.
    pub icon: PathBuf,
    /// Arbitrary tags used for store search and filtering.
    pub tags: Vec<String>,
    /// Which kind of resource this is.
    pub resource_type: ResourceType,
    /// Other resources this package requires before it can be installed.
    pub dependencies: Vec<Dependency>,
    /// Base64-encoded ed25519 signature over the package content hash.
    /// `None` for unsigned (local / development) packages.
    pub signature: Option<String>,
    /// Continuously-visible validation status — computed on every load.
    pub status: ValidationStatus,
    /// Upstream provenance: registry, image, Git repo and website.
    /// `None` for native FSN resources (apps, widgets, themes, …).
    pub source: Option<PackageSource>,
    /// Platform and feature requirements for installation.
    /// `None` means compatible with all platforms.
    #[serde(default)]
    pub platform: Option<PlatformFilter>,
}

impl ResourceMeta {
    /// Run basic field validation and return an updated status.
    ///
    /// This does **not** verify the cryptographic signature — that requires
    /// the `fs-crypto` crate.  It only checks structural completeness.
    pub fn validate(&mut self) {
        let broken = self.id.trim().is_empty()
            || self.name.trim().is_empty()
            || !self.icon.extension().is_some_and(|e| e.eq_ignore_ascii_case("svg"));

        if broken {
            self.status = ValidationStatus::Broken;
            return;
        }

        let incomplete = self.description.trim().len() < 10
            || self.tags.is_empty()
            || self.author.trim().is_empty()
            || self.license.trim().is_empty();

        self.status = if incomplete {
            ValidationStatus::Incomplete
        } else {
            ValidationStatus::Ok
        };
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn base_meta(id: &str) -> ResourceMeta {
        ResourceMeta {
            id: id.into(),
            name: "Test Resource".into(),
            description: "A sufficiently long description.".into(),
            version: "1.0.0".into(),
            author: "FreeSynergy".into(),
            license: "MIT".into(),
            icon: PathBuf::from("icon.svg"),
            tags: vec!["test".into()],
            resource_type: ResourceType::App,
            dependencies: vec![],
            signature: None,
            status: ValidationStatus::Incomplete,
            source: None,
            platform: None,
        }
    }

    #[test]
    fn validate_complete_meta_is_ok() {
        let mut m = base_meta("my-app");
        m.validate();
        assert_eq!(m.status, ValidationStatus::Ok);
    }

    #[test]
    fn validate_empty_id_is_broken() {
        let mut m = base_meta("");
        m.validate();
        assert_eq!(m.status, ValidationStatus::Broken);
    }

    #[test]
    fn validate_non_svg_icon_is_broken() {
        let mut m = base_meta("app");
        m.icon = PathBuf::from("icon.png");
        m.validate();
        assert_eq!(m.status, ValidationStatus::Broken);
    }

    #[test]
    fn validate_short_description_is_incomplete() {
        let mut m = base_meta("app");
        m.description = "Short".into();
        m.validate();
        assert_eq!(m.status, ValidationStatus::Incomplete);
    }

    #[test]
    fn validation_status_badge() {
        assert_eq!(ValidationStatus::Ok.badge(), "✅");
        assert_eq!(ValidationStatus::Incomplete.badge(), "⚠️");
        assert_eq!(ValidationStatus::Broken.badge(), "❌");
    }

    #[test]
    fn resource_type_is_theme_component() {
        assert!(ResourceType::ColorScheme.is_theme_component());
        assert!(ResourceType::Style.is_theme_component());
        assert!(!ResourceType::App.is_theme_component());
        assert!(!ResourceType::Container.is_theme_component());
    }

    #[test]
    fn role_display() {
        let r = Role::new("iam");
        assert_eq!(r.to_string(), "iam");
    }
}
