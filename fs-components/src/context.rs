/// AppContext — single injectable context for all cross-cutting desktop concerns.
///
/// Provided once at the Desktop root via `use_context_provider`.
/// Every component that needs locale, theme, or appearance settings
/// calls `use_context::<AppContext>()` — no Props drilling, no per-component fallbacks.
///
/// # Usage
///
/// **Root (Desktop):**
/// ```rust
/// use_context_provider(|| AppContext {
///     locale:        Signal::new("en".to_string()),
///     theme:         Signal::new("midnight-blue".to_string()),
///     ..Default::default()
/// });
/// ```
///
/// **Any child component:**
/// ```rust
/// let ctx = use_context::<AppContext>();
/// let lang = ctx.locale.read().clone();
/// ```
use dioxus::prelude::*;

/// Desktop-wide application context.
///
/// All fields are `Signal<T>` — writing to them triggers reactive re-renders
/// in all components that have called `use_context::<AppContext>()`.
#[derive(Clone, Copy)]
pub struct AppContext {
    /// Active locale code, e.g. `"en"` or `"de"`.
    /// Write to trigger a global language change.
    pub locale: Signal<String>,

    /// Active theme identifier.
    /// Built-in: `"midnight-blue"`.
    /// Store theme: `"__custom__<css>"` (Desktop strips prefix and injects CSS).
    pub theme: Signal<String>,

    /// Wallpaper CSS background string.
    /// e.g. `"linear-gradient(...)"` or `"url(...) center/cover no-repeat"`.
    pub wallpaper: Signal<String>,

    /// Whether UI animations are enabled.
    /// Controls `--fs-anim-duration`: `180ms` when true, `0ms` when false.
    pub anim_enabled: Signal<bool>,

    /// Window chrome background opacity (0.0–1.0).
    /// Controls `--fs-window-bg` alpha channel.
    pub chrome_opacity: Signal<f64>,

    /// Window chrome button style: `"kde"` | `"macos"` | `"windows"` | `"minimal"`.
    /// Applied as `data-chrome-style` on the root element.
    pub chrome_style: Signal<String>,

    /// Button border-radius style: `"rounded"` | `"square"` | `"pill"` | `"flat"`.
    /// Applied as `data-btn-style` on the root element.
    pub btn_style: Signal<String>,

    /// Sidebar background style: `"solid"` | `"glass"` | `"transparent"`.
    /// Applied as `data-sidebar-style` on the root element.
    pub sidebar_style: Signal<String>,

    /// Inter-app navigation request.
    /// A sub-app writes `Some(app_id)` to request the Desktop open another app.
    /// The Desktop reads this in a `use_effect`, handles it, and resets to `None`.
    pub app_open_req: Signal<Option<String>>,
}
