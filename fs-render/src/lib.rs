// fs-render — rendering abstractions for FreeSynergy.
//
// Provides two independent layers:
//
//   1. `RenderCtx` — renderer-agnostic context (theme, locale, feature flags).
//      Always compiled. Business logic passes this around without importing Dioxus.
//
//   2. `DioxusView` trait — Dioxus-specific view interface.
//      Compiled only when the `dioxus` feature is enabled.
//      Domain objects implement `DioxusView` in their UI crates to return
//      a Dioxus `Element` from `fn view(&self) -> Element`.
//
// Feature flags:
//   dioxus — enables DioxusView trait

pub use fs_config::FeatureFlags;
pub use fs_theme::Theme;

// ── UserEvent ─────────────────────────────────────────────────────────────────

/// Normalised user input event, independent of the rendering backend.
#[derive(Debug, Clone, PartialEq)]
pub enum UserEvent {
    /// A keyboard key was pressed.
    Key(KeyEvent),
    /// A mouse click or tap at the given coordinates.
    Click {
        x: f64,
        y: f64,
    },
    /// A text change from an input element.
    TextChange(String),
    /// A navigation intent (tab focus order).
    FocusNext,
    FocusPrev,
    /// Application-defined action string (e.g. `"submit"`, `"cancel"`).
    Action(String),
}

/// A keyboard event with the key name and modifier flags.
#[derive(Debug, Clone, PartialEq)]
pub struct KeyEvent {
    /// Key name, e.g. `"Enter"`, `"Escape"`, `"Tab"`, `"a"`.
    pub key: String,
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
}

impl KeyEvent {
    /// Convenience constructor for a plain key (no modifiers).
    pub fn plain(key: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            ctrl: false,
            alt: false,
            shift: false,
        }
    }
}

// ── RenderCtx ────────────────────────────────────────────────────────────────

/// Renderer-agnostic context for cross-cutting concerns.
///
/// Passed to business logic that needs to know the current theme or locale
/// without depending on Dioxus directly.
///
/// In Dioxus apps this data is also available via `AppContext` from `fs-components`,
/// which provides reactive `Signal<T>` versions of the same fields.
pub struct RenderCtx {
    /// Active theme (colors, typography, shadows, glass params).
    pub theme: Theme,
    /// Active locale tag, e.g. `"en"` or `"de"`.
    pub locale: String,
    /// Named feature flags (runtime-configurable).
    pub features: FeatureFlags,
}

impl Default for RenderCtx {
    fn default() -> Self {
        Self {
            theme: Theme::default(),
            locale: "en".into(),
            features: FeatureFlags::default(),
        }
    }
}

// ── DioxusView ────────────────────────────────────────────────────────────────

#[cfg(feature = "dioxus")]
pub mod dioxus_view {
    use dioxus::prelude::Element;

    /// Dioxus view interface for domain objects.
    ///
    /// Implement this trait on a domain object in its UI crate
    /// (where Dioxus is already a dependency) to give the object
    /// the ability to render itself as a Dioxus `Element`.
    ///
    /// # Example
    ///
    /// ```rust
    /// // In fs-settings (has dioxus dep):
    /// use fs_render::dioxus_view::DioxusView;
    /// use dioxus::prelude::*;
    ///
    /// impl DioxusView for LanguageManager {
    ///     fn view(&self) -> Element {
    ///         rsx! { div { "Language: {self.active_locale()}" } }
    ///     }
    /// }
    /// ```
    ///
    /// Then call `{manager.view()}` inside any `rsx!` block.
    pub trait DioxusView {
        /// Render the object as a Dioxus element.
        fn view(&self) -> Element;
    }
}

#[cfg(feature = "dioxus")]
pub use dioxus_view::DioxusView;

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_ctx_default_is_english() {
        let ctx = RenderCtx::default();
        assert_eq!(ctx.locale, "en");
        assert_eq!(ctx.theme.name, "FreeSynergy Default");
    }

    #[test]
    fn feature_flags_disabled_by_default() {
        let flags = FeatureFlags::default();
        assert!(!flags.is_enabled("anything"));
    }

    #[test]
    fn feature_flags_can_be_set() {
        let mut flags = FeatureFlags::default();
        flags.enable("experimental_ui");
        assert!(flags.is_enabled("experimental_ui"));
        assert!(!flags.is_enabled("other_flag"));
    }

    #[test]
    fn key_event_plain_has_no_modifiers() {
        let k = KeyEvent::plain("Enter");
        assert_eq!(k.key, "Enter");
        assert!(!k.ctrl);
        assert!(!k.alt);
        assert!(!k.shift);
    }
}
