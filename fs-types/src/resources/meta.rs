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

    /// The minimum set of standard methods that a bridge for this role must map.
    ///
    /// A bridge that is missing any of these gets `ValidationStatus::Incomplete`.
    pub fn required_bridge_methods(&self) -> &'static [&'static str] {
        match self.0.as_str() {
            "iam" => &[
                "user.create",
                "user.get",
                "user.list",
                "user.update",
                "user.delete",
                "group.create",
                "group.list",
                "group.add_member",
            ],
            "wiki" => &["page.create", "page.get", "page.list", "page.search"],
            "git" => &["repo.create", "repo.list", "repo.get", "commit.list"],
            "chat" => &["message.send", "channel.list", "channel.get"],
            "database" => &["query.execute", "schema.list"],
            "cache" => &["key.get", "key.set", "key.delete"],
            "smtp" => &["mail.send"],
            "llm" => &["completion.create", "model.list"],
            "map" => &["tile.get", "search.geocode"],
            "tasks" => &["task.create", "task.list", "task.update"],
            "monitoring" => &["metric.query", "alert.list"],
            _ => &[],
        }
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

/// Discriminant for all resource kinds a FreeSynergy store can contain.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResourceType {
    // ── Programs ──────────────────────────────────────────────────────────────
    /// A native FreeSynergy binary (Node, Desktop, Kanidm, Stalwart, Zentinel, Mistral, …).
    /// Binaries are not stored here — only metadata + references to GitHub Releases.
    App,
    /// A containerised application (Forgejo, Postgres, Outline, …).
    /// Runtime-agnostic: runs with Podman or Docker.
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
    /// Shared Mozilla Fluent snippets (save, cancel, error, …) used by all programs.
    /// Program-specific translations are bundled inside each program's own package.
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
    /// SVG icon collection — can override the default FreeSynergy icon set.
    IconSet,
    /// Button appearance (shape, border, padding, hover effect).
    ButtonStyle,
    /// Window title-bar style.
    WindowChrome,
    /// CSS transitions and keyframe animations.
    AnimationSet,
    /// A messenger platform adapter — implements the `Channel` trait for one platform.
    MessengerAdapter,

    // ── Store infrastructure ───────────────────────────────────────────────────
    /// Bootstrap binary — not installable, only downloadable.
    /// Used for fs-init: the binary that bootstraps the Store on a new machine.
    Bootstrap,
    /// An additional package repository source.
    /// Installing a `Repo` package registers a new catalog URL as a trusted source.
    Repo,
    /// A theme bundle — a Bundle subtype with a fixed structure: color scheme,
    /// style, icon set, font set, cursor set, button style, window chrome,
    /// and animation set. Lives at the root-level `themes/` directory.
    Theme,
}

impl ResourceType {
    /// Human-readable label for UI display.
    pub fn label(self) -> &'static str {
        match self {
            ResourceType::App => "App",
            ResourceType::Container => "Container",
            ResourceType::Bundle => "Bundle",
            ResourceType::Widget => "Widget",
            ResourceType::Bot => "Bot",
            ResourceType::Bridge => "Bridge",
            ResourceType::Task => "Task",
            ResourceType::Language => "Language",
            ResourceType::ColorScheme => "Color Scheme",
            ResourceType::Style => "Style",
            ResourceType::FontSet => "Font Set",
            ResourceType::CursorSet => "Cursor Set",
            ResourceType::IconSet => "Icon Set",
            ResourceType::ButtonStyle => "Button Style",
            ResourceType::WindowChrome => "Window Chrome",
            ResourceType::AnimationSet => "Animation Set",
            ResourceType::MessengerAdapter => "Messenger Adapter",
            ResourceType::Bootstrap => "Bootstrap",
            ResourceType::Repo => "Repository",
            ResourceType::Theme => "Theme",
        }
    }

    /// i18n key for translations.
    pub fn i18n_key(self) -> &'static str {
        match self {
            ResourceType::App => "resource.type.app",
            ResourceType::Container => "resource.type.container",
            ResourceType::Bundle => "resource.type.bundle",
            ResourceType::Widget => "resource.type.widget",
            ResourceType::Bot => "resource.type.bot",
            ResourceType::Bridge => "resource.type.bridge",
            ResourceType::Task => "resource.type.task",
            ResourceType::Language => "resource.type.language",
            ResourceType::ColorScheme => "resource.type.color_scheme",
            ResourceType::Style => "resource.type.style",
            ResourceType::FontSet => "resource.type.font_set",
            ResourceType::CursorSet => "resource.type.cursor_set",
            ResourceType::IconSet => "resource.type.icon_set",
            ResourceType::ButtonStyle => "resource.type.button_style",
            ResourceType::WindowChrome => "resource.type.window_chrome",
            ResourceType::AnimationSet => "resource.type.animation_set",
            ResourceType::MessengerAdapter => "resource.type.messenger_adapter",
            ResourceType::Bootstrap => "resource.type.bootstrap",
            ResourceType::Repo => "resource.type.repo",
            ResourceType::Theme => "resource.type.theme",
        }
    }

    /// `true` when this resource is a bundle — either a generic `Bundle` or a `Theme`.
    ///
    /// Both live at root level in the Store (not under `packages/`) and reference
    /// other resources by id rather than containing packages themselves.
    pub fn is_bundle(self) -> bool {
        matches!(self, ResourceType::Bundle | ResourceType::Theme)
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
            ValidationStatus::Ok => "✅",
            ValidationStatus::Incomplete => "⚠️",
            ValidationStatus::Broken => "❌",
        }
    }

    /// i18n key.
    pub fn i18n_key(self) -> &'static str {
        match self {
            ValidationStatus::Ok => "validation.ok",
            ValidationStatus::Incomplete => "validation.incomplete",
            ValidationStatus::Broken => "validation.broken",
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
        Self {
            id: id.into(),
            version_req: version_req.into(),
            optional: false,
        }
    }

    /// Create an optional dependency.
    pub fn optional(id: impl Into<String>, version_req: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            version_req: version_req.into(),
            optional: true,
        }
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
///
/// ## Description levels
///
/// Every resource has three levels of description:
///
/// | Field              | Max length | Where used                                      |
/// |--------------------|------------|-------------------------------------------------|
/// | `summary`          | 255 chars  | Store card, search results, sidebar             |
/// | `description`      | free text  | Store detail view (inline in catalog)           |
/// | `description_file` | path       | Long `.ftl` file — documentation, help pages   |
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceMeta {
    /// Unique slug identifier (URL-safe, lowercase, hyphens), e.g. `"kanidm"`.
    pub id: String,
    /// Human-readable display name, e.g. `"Kanidm"`.
    pub name: String,
    /// One-line summary shown in store listings and search results (mandatory, max 255 chars).
    pub summary: String,
    /// Medium-length description shown in the store detail view. Inline in the catalog.
    /// Mandatory — if missing, the validator reports `ValidationStatus::Broken`.
    pub description: String,
    /// Path to the `.ftl` file containing the long-form description.
    /// Internationalised: `help/en/description.ftl`, `help/de/description.ftl`, …
    /// Mandatory — every package must ship a translatable long description.
    pub description_file: PathBuf,
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
    /// Builder: set `name`.
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }

    /// Builder: set `summary` (shown in store listings, max 255 chars).
    pub fn with_summary(mut self, summary: impl Into<String>) -> Self {
        self.summary = summary.into();
        self
    }

    /// Builder: set `description` (medium, shown in detail view).
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }

    /// Builder: set `description_file` (path to long `.ftl` description).
    pub fn with_description_file(mut self, path: impl Into<PathBuf>) -> Self {
        self.description_file = path.into();
        self
    }

    /// Builder: set `version`.
    pub fn with_version(mut self, version: impl Into<String>) -> Self {
        self.version = version.into();
        self
    }

    /// Builder: set `tags` (replaces existing).
    pub fn with_tags(mut self, tags: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.tags = tags.into_iter().map(|t| t.into()).collect();
        self
    }

    /// Run basic field validation and return an updated status.
    ///
    /// This does **not** verify the cryptographic signature — that requires
    /// the `fs-crypto` crate.  It only checks structural completeness.
    pub fn validate(&mut self) {
        let broken = self.id.trim().is_empty()
            || self.name.trim().is_empty()
            || !self
                .icon
                .extension()
                .is_some_and(|e| e.eq_ignore_ascii_case("svg"))
            || self.description.trim().is_empty()
            || self.description_file.as_os_str().is_empty();

        if broken {
            self.status = ValidationStatus::Broken;
            return;
        }

        let incomplete = self.summary.trim().len() < 10
            || self.summary.len() > 255
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
            summary: "A sufficiently long summary for store listings.".into(),
            description: "A medium-length description shown in the store detail view.".into(),
            description_file: PathBuf::from("help/en/description.ftl"),
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
    fn validate_short_summary_is_incomplete() {
        let mut m = base_meta("app");
        m.summary = "Short".into();
        m.validate();
        assert_eq!(m.status, ValidationStatus::Incomplete);
    }

    #[test]
    fn validate_summary_over_255_is_incomplete() {
        let mut m = base_meta("app");
        m.summary = "x".repeat(256);
        m.validate();
        assert_eq!(m.status, ValidationStatus::Incomplete);
    }

    #[test]
    fn resource_type_labels_all_variants() {
        assert_eq!(ResourceType::App.label(), "App");
        assert_eq!(ResourceType::Container.label(), "Container");
        assert_eq!(ResourceType::Bundle.label(), "Bundle");
        assert_eq!(ResourceType::Widget.label(), "Widget");
        assert_eq!(ResourceType::Bot.label(), "Bot");
        assert_eq!(ResourceType::Bridge.label(), "Bridge");
        assert_eq!(ResourceType::Task.label(), "Task");
        assert_eq!(ResourceType::Language.label(), "Language");
        assert_eq!(ResourceType::ColorScheme.label(), "Color Scheme");
        assert_eq!(ResourceType::Style.label(), "Style");
        assert_eq!(ResourceType::FontSet.label(), "Font Set");
        assert_eq!(ResourceType::CursorSet.label(), "Cursor Set");
        assert_eq!(ResourceType::IconSet.label(), "Icon Set");
        assert_eq!(ResourceType::ButtonStyle.label(), "Button Style");
        assert_eq!(ResourceType::WindowChrome.label(), "Window Chrome");
        assert_eq!(ResourceType::AnimationSet.label(), "Animation Set");
        assert_eq!(ResourceType::MessengerAdapter.label(), "Messenger Adapter");
        assert_eq!(ResourceType::Bootstrap.label(), "Bootstrap");
        assert_eq!(ResourceType::Repo.label(), "Repository");
        assert_eq!(ResourceType::Theme.label(), "Theme");
    }

    #[test]
    fn resource_type_i18n_keys_all_variants() {
        assert_eq!(ResourceType::App.i18n_key(), "resource.type.app");
        assert_eq!(
            ResourceType::Container.i18n_key(),
            "resource.type.container"
        );
        assert_eq!(ResourceType::Bundle.i18n_key(), "resource.type.bundle");
        assert_eq!(ResourceType::Widget.i18n_key(), "resource.type.widget");
        assert_eq!(ResourceType::Bot.i18n_key(), "resource.type.bot");
        assert_eq!(ResourceType::Bridge.i18n_key(), "resource.type.bridge");
        assert_eq!(ResourceType::Task.i18n_key(), "resource.type.task");
        assert_eq!(ResourceType::Language.i18n_key(), "resource.type.language");
        assert_eq!(
            ResourceType::ColorScheme.i18n_key(),
            "resource.type.color_scheme"
        );
        assert_eq!(ResourceType::Style.i18n_key(), "resource.type.style");
        assert_eq!(ResourceType::FontSet.i18n_key(), "resource.type.font_set");
        assert_eq!(
            ResourceType::CursorSet.i18n_key(),
            "resource.type.cursor_set"
        );
        assert_eq!(ResourceType::IconSet.i18n_key(), "resource.type.icon_set");
        assert_eq!(
            ResourceType::ButtonStyle.i18n_key(),
            "resource.type.button_style"
        );
        assert_eq!(
            ResourceType::WindowChrome.i18n_key(),
            "resource.type.window_chrome"
        );
        assert_eq!(
            ResourceType::AnimationSet.i18n_key(),
            "resource.type.animation_set"
        );
        assert_eq!(
            ResourceType::MessengerAdapter.i18n_key(),
            "resource.type.messenger_adapter"
        );
        assert_eq!(
            ResourceType::Bootstrap.i18n_key(),
            "resource.type.bootstrap"
        );
        assert_eq!(ResourceType::Repo.i18n_key(), "resource.type.repo");
        assert_eq!(ResourceType::Theme.i18n_key(), "resource.type.theme");
    }

    #[test]
    fn resource_type_is_bundle() {
        // Bundle and Theme are both bundles (root-level, reference other packages)
        assert!(ResourceType::Bundle.is_bundle());
        assert!(ResourceType::Theme.is_bundle());
        // Everything else is not a bundle
        assert!(!ResourceType::App.is_bundle());
        assert!(!ResourceType::Container.is_bundle());
        assert!(!ResourceType::Widget.is_bundle());
        assert!(!ResourceType::ColorScheme.is_bundle());
        assert!(!ResourceType::Bootstrap.is_bundle());
        assert!(!ResourceType::Repo.is_bundle());
    }

    #[test]
    fn validation_status_badge() {
        assert_eq!(ValidationStatus::Ok.badge(), "✅");
        assert_eq!(ValidationStatus::Incomplete.badge(), "⚠️");
        assert_eq!(ValidationStatus::Broken.badge(), "❌");
    }

    #[test]
    fn resource_type_is_theme_component() {
        // All theme component variants
        assert!(ResourceType::ColorScheme.is_theme_component());
        assert!(ResourceType::Style.is_theme_component());
        assert!(ResourceType::FontSet.is_theme_component());
        assert!(ResourceType::CursorSet.is_theme_component());
        assert!(ResourceType::IconSet.is_theme_component());
        assert!(ResourceType::ButtonStyle.is_theme_component());
        assert!(ResourceType::WindowChrome.is_theme_component());
        assert!(ResourceType::AnimationSet.is_theme_component());
        // Non-theme-component variants — including Bundle and Theme
        assert!(!ResourceType::App.is_theme_component());
        assert!(!ResourceType::Container.is_theme_component());
        assert!(!ResourceType::Bundle.is_theme_component());
        assert!(!ResourceType::Theme.is_theme_component());
        assert!(!ResourceType::Widget.is_theme_component());
        assert!(!ResourceType::Bootstrap.is_theme_component());
        assert!(!ResourceType::Repo.is_theme_component());
    }

    #[test]
    fn validate_bundle_type_meta_is_ok() {
        let mut m = base_meta("zentinel");
        m.resource_type = ResourceType::Bundle;
        m.validate();
        assert_eq!(m.status, ValidationStatus::Ok);
    }

    #[test]
    fn validate_theme_type_meta_is_ok() {
        let mut m = base_meta("midnight-blue");
        m.resource_type = ResourceType::Theme;
        m.validate();
        assert_eq!(m.status, ValidationStatus::Ok);
    }

    #[test]
    fn role_display() {
        let r = Role::new("iam");
        assert_eq!(r.to_string(), "iam");
    }
}
