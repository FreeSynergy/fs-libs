// fsn-render — ViewRenderer abstraction for FreeSynergy.
//
// Defines the `ViewRenderer` trait as a common interface for all rendering
// backends (Desktop/Web via Dioxus, future mobile).
//
// Business logic that implements `ViewRenderer` stays renderer-agnostic;
// the concrete renderer is injected at startup.
//
// Feature flags:
//   dioxus — DioxusRenderer marker type

use std::time::Duration;

pub use fsn_config::FeatureFlags;
pub use fsn_theme::Theme;

// ── UserEvent ─────────────────────────────────────────────────────────────────

/// Normalised user input event, independent of the rendering backend.
#[derive(Debug, Clone, PartialEq)]
pub enum UserEvent {
    /// A keyboard key was pressed.
    Key(KeyEvent),
    /// A mouse click or tap at the given coordinates.
    Click { x: f64, y: f64 },
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
    pub key:   String,
    pub ctrl:  bool,
    pub alt:   bool,
    pub shift: bool,
}

impl KeyEvent {
    /// Convenience constructor for a plain key (no modifiers).
    pub fn plain(key: impl Into<String>) -> Self {
        Self { key: key.into(), ctrl: false, alt: false, shift: false }
    }
}

// ── RenderCtx ────────────────────────────────────────────────────────────────

/// Injected render context — available to every `ViewRenderer::render` call.
///
/// Carries cross-cutting concerns that are injected, never global.
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
            theme:    Theme::default(),
            locale:   "en".into(),
            features: FeatureFlags::default(),
        }
    }
}

// ── ViewRenderer trait ────────────────────────────────────────────────────────

/// Common rendering interface for all FreeSynergy views.
///
/// Implement this for any view that should work across renderers:
/// - Desktop/Web (Dioxus): implement `render` by returning RSX elements or
///   signalling a reactive re-render.
/// - Tests: implement `render` as a no-op or string collector.
///
/// # Lifecycle
/// ```text
/// loop {
///     renderer.handle_event(event)?;   // process input
///     changed = renderer.update(dt);   // advance state
///     if changed { renderer.render(ctx); }  // paint
/// }
/// ```
pub trait ViewRenderer {
    /// Paint the current state. Called after `update` returns `true` or on
    /// the first frame.
    fn render(&self, ctx: &RenderCtx);

    /// Process a normalised input event. Returns `true` if the event was
    /// consumed (prevents further propagation).
    fn handle_event(&mut self, event: UserEvent) -> bool;

    /// Advance internal animation or async state. `delta` is the time since
    /// the last call. Returns `true` if a repaint is needed.
    fn update(&mut self, delta: Duration) -> bool;
}

// ── DioxusRenderer ────────────────────────────────────────────────────────────

/// Desktop / Web renderer backed by Dioxus.
///
/// Dioxus manages its own reactive graph and diffing; the `render` method
/// is a thin hook into the Dioxus signal system.
#[cfg(feature = "dioxus")]
pub mod dioxus_renderer {
    use super::{RenderCtx, UserEvent, ViewRenderer};
    use std::time::Duration;

    /// Marker renderer for Dioxus-based views.
    ///
    /// In practice, Dioxus components are functions — not objects. This struct
    /// acts as a bridge for business-logic objects that need to interact with
    /// the renderer without depending on Dioxus's `#[component]` macro.
    ///
    /// Use `DioxusRenderer::signal_repaint()` to trigger a Dioxus re-render
    /// via a shared signal.
    pub struct DioxusRenderer {
        needs_repaint: bool,
    }

    impl Default for DioxusRenderer {
        fn default() -> Self {
            Self { needs_repaint: true }
        }
    }

    impl DioxusRenderer {
        pub fn new() -> Self {
            Self::default()
        }
    }

    impl ViewRenderer for DioxusRenderer {
        fn render(&self, _ctx: &RenderCtx) {
            // Dioxus renders reactively via signals. This method is called
            // to trigger a re-render signal from outside the reactive graph.
        }

        fn handle_event(&mut self, event: UserEvent) -> bool {
            self.needs_repaint = true;
            let _ = event;
            false
        }

        fn update(&mut self, _delta: Duration) -> bool {
            let dirty = self.needs_repaint;
            self.needs_repaint = false;
            dirty
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    struct NoopRenderer { repaint: bool }

    impl ViewRenderer for NoopRenderer {
        fn render(&self, _ctx: &RenderCtx) {}
        fn handle_event(&mut self, _event: UserEvent) -> bool { self.repaint = true; false }
        fn update(&mut self, _delta: Duration) -> bool {
            let v = self.repaint; self.repaint = false; v
        }
    }

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
    fn view_renderer_update_clears_dirty() {
        let mut r = NoopRenderer { repaint: true };
        assert!(r.update(Duration::ZERO));
        assert!(!r.update(Duration::ZERO));
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
