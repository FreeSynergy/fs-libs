// Desktop launch abstraction.
//
// ALL dioxus::desktop API calls (LaunchBuilder, Config, WindowBuilder,
// window().new_window()) are isolated here.
//
// When the Dioxus desktop API changes, only this file needs updating.
// Callers use `DesktopConfig`, `launch_desktop`, and `spawn_window` —
// no dioxus imports in their code.

use dioxus::prelude::Element;

// ── DesktopConfig ─────────────────────────────────────────────────────────────

/// Configuration for a FreeSynergy desktop window.
///
/// Build with the builder methods, then pass to `launch_desktop` or `spawn_window`.
/// This is a pure domain type — no dioxus imports at the call site.
#[derive(Clone, Default)]
pub struct DesktopConfig {
    title:          Option<String>,
    width:          Option<f64>,
    height:         Option<f64>,
    min_size:       Option<(f64, f64)>,
    decorations:    bool,
    resizable:      bool,
    background:     Option<(u8, u8, u8, u8)>,
    all_navigation: bool,
}

impl DesktopConfig {
    pub fn new() -> Self {
        Self { decorations: true, resizable: true, ..Default::default() }
    }

    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into()); self
    }

    pub fn with_size(mut self, width: f64, height: f64) -> Self {
        self.width = Some(width); self.height = Some(height); self
    }

    pub fn with_min_size(mut self, width: f64, height: f64) -> Self {
        self.min_size = Some((width, height)); self
    }

    pub fn without_decorations(mut self) -> Self {
        self.decorations = false; self
    }

    pub fn without_resizable(mut self) -> Self {
        self.resizable = false; self
    }

    pub fn with_background(mut self, r: u8, g: u8, b: u8, a: u8) -> Self {
        self.background = Some((r, g, b, a)); self
    }

    /// Allow all WebView navigations (needed for the Browser app to load external URLs
    /// in iframes instead of opening the system browser).
    /// Tracking PR: https://github.com/DioxusLabs/dioxus/pull/5390
    pub fn with_all_navigation(mut self) -> Self {
        self.all_navigation = true; self
    }

    /// Convert into a Dioxus desktop `Config`.
    /// This is the ONLY place that imports `dioxus::desktop` types.
    fn into_dioxus_cfg(self) -> dioxus::desktop::Config {
        use dioxus::desktop::{Config, LogicalSize, WindowBuilder};

        let mut wb = WindowBuilder::new()
            .with_decorations(self.decorations)
            .with_resizable(self.resizable);

        if let Some(title) = self.title {
            wb = wb.with_title(title);
        }
        if let (Some(w), Some(h)) = (self.width, self.height) {
            wb = wb.with_inner_size(LogicalSize::new(w, h));
        }
        if let Some((min_w, min_h)) = self.min_size {
            wb = wb.with_min_inner_size(LogicalSize::new(min_w, min_h));
        }

        let mut cfg = Config::new().with_window(wb);

        if let Some((r, g, b, a)) = self.background {
            cfg = cfg.with_background_color((r, g, b, a));
        }
        if self.all_navigation {
            cfg = cfg.with_navigation_handler(|_url: String| true);
        }

        cfg
    }
}

// ── Public API ────────────────────────────────────────────────────────────────

/// Launches a desktop app. Call once from `main()`.
///
/// This is the single entry point for all `dioxus::LaunchBuilder::desktop()` calls.
pub fn launch_desktop(config: DesktopConfig, app: fn() -> Element) {
    dioxus::LaunchBuilder::desktop()
        .with_cfg(config.into_dioxus_cfg())
        .launch(app);
}

/// Spawns a new OS window from within a running Dioxus component context.
///
/// This is the single entry point for all `window().new_window()` calls.
pub fn spawn_window(config: DesktopConfig, component: fn() -> Element) {
    use dioxus::prelude::VirtualDom;
    dioxus::desktop::window().new_window(VirtualDom::new(component), config.into_dioxus_cfg());
}
