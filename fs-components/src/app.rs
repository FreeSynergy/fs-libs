// fs-components/app.rs — AppLauncher, ThemeSwitcher, LangSwitcher.

use dioxus::prelude::*;

// ── AppEntry ──────────────────────────────────────────────────────────────────

/// A launchable application entry for `AppLauncher`.
#[derive(Clone, PartialEq)]
pub struct AppEntry {
    /// Unique application identifier emitted by `on_launch`.
    pub id: String,
    /// Display name.
    pub name: String,
    /// Icon (emoji, short text, or URL depending on rendering context).
    pub icon: String,
    /// Optional one-line description.
    pub description: Option<String>,
}

impl AppEntry {
    /// Construct an `AppEntry` without a description.
    pub fn new(id: impl Into<String>, name: impl Into<String>, icon: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            icon: icon.into(),
            description: None,
        }
    }

    /// Attach a description.
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }
}

// ── AppLauncher ───────────────────────────────────────────────────────────────

#[derive(Props, Clone, PartialEq)]
pub struct AppLauncherProps {
    /// Applications to display.
    pub entries: Vec<AppEntry>,
    /// Called with the `id` of the clicked application.
    #[props(default)]
    pub on_launch: EventHandler<String>,
}

/// Grid of application cards. Each card shows an icon, name and optional
/// description. Clicking a card fires `on_launch` with the app's `id`.
///
/// ```no_run
/// AppLauncher {
///     entries: apps.read().clone(),
///     on_launch: move |id| navigate(id),
/// }
/// ```
#[component]
pub fn AppLauncher(props: AppLauncherProps) -> Element {
    rsx! {
        div {
            class: "fs-app-launcher",
            style: "display: grid; \
                    grid-template-columns: repeat(auto-fill, minmax(120px, 1fr)); \
                    gap: 12px;",

            for entry in &props.entries {
                {
                    let id      = entry.id.clone();
                    let name    = entry.name.clone();
                    let icon    = entry.icon.clone();
                    let desc    = entry.description.clone();
                    let trigger = props.on_launch.clone();

                    rsx! {
                        button {
                            key: "{id}",
                            class: "fs-app-card",
                            style: "display: flex; flex-direction: column; align-items: center; \
                                    gap: 8px; padding: 16px 12px; border-radius: 10px; \
                                    background: var(--fs-color-bg-surface, #1e293b); \
                                    border: 1px solid var(--fs-color-border-default, #334155); \
                                    cursor: pointer; font-family: inherit; \
                                    transition: background 0.15s, border-color 0.15s;",
                            onclick: move |_| trigger.call(id.clone()),

                            span {
                                aria_hidden: "true",
                                style: "font-size: 28px; line-height: 1;",
                                "{icon}"
                            }
                            span {
                                style: "font-size: 12px; font-weight: 600; \
                                        color: var(--fs-color-text-primary, #e2e8f0); \
                                        text-align: center;",
                                "{name}"
                            }
                            if let Some(d) = &desc {
                                span {
                                    style: "font-size: 11px; \
                                            color: var(--fs-color-text-muted, #64748b); \
                                            text-align: center; line-height: 1.4;",
                                    "{d}"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

// ── ThemeOption ───────────────────────────────────────────────────────────────

/// A selectable theme for `ThemeSwitcher`.
#[derive(Clone, PartialEq)]
pub struct ThemeOption {
    /// Unique identifier emitted by `on_change`.
    pub id: String,
    /// Human-readable theme name.
    pub name: String,
    /// Hex colour used as the swatch preview (e.g. `"#06b6d4"`).
    pub preview_color: String,
}

impl ThemeOption {
    /// Construct a theme option.
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        preview_color: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            preview_color: preview_color.into(),
        }
    }
}

// ── ThemeSwitcher ─────────────────────────────────────────────────────────────

#[derive(Props, Clone, PartialEq)]
pub struct ThemeSwitcherProps {
    /// Available themes.
    pub themes: Vec<ThemeOption>,
    /// Currently active theme id.
    pub active: String,
    /// Called with the newly selected theme id.
    #[props(default)]
    pub on_change: EventHandler<String>,
}

/// Row of colour swatches with labels for selecting the active theme.
///
/// ```no_run
/// ThemeSwitcher {
///     themes: vec![ThemeOption::new("cyan", "Cyan", "#06b6d4")],
///     active: active_theme.read().clone(),
///     on_change: move |id| active_theme.set(id),
/// }
/// ```
#[component]
pub fn ThemeSwitcher(props: ThemeSwitcherProps) -> Element {
    rsx! {
        div {
            class: "fs-theme-switcher",
            style: "display: flex; flex-wrap: wrap; gap: 12px; align-items: center;",

            for theme in &props.themes {
                {
                    let id        = theme.id.clone();
                    let name      = theme.name.clone();
                    let color     = theme.preview_color.clone();
                    let is_active = theme.id == props.active;
                    let trigger   = props.on_change.clone();

                    let ring = if is_active {
                        "box-shadow: 0 0 0 2px #fff, 0 0 0 4px {color};"
                    } else {
                        ""
                    };

                    let label_color = if is_active {
                        "var(--fs-color-text-primary, #e2e8f0)"
                    } else {
                        "var(--fs-color-text-muted, #64748b)"
                    };

                    rsx! {
                        button {
                            key: "{id}",
                            aria_label: "Switch to {name} theme",
                            aria_pressed: if is_active { "true" } else { "false" },
                            style: "display: flex; flex-direction: column; align-items: center; \
                                    gap: 5px; background: none; border: none; cursor: pointer; \
                                    padding: 4px; border-radius: 8px;",
                            onclick: move |_| trigger.call(id.clone()),

                            span {
                                style: "display: block; width: 28px; height: 28px; \
                                        border-radius: 50%; background: {color}; {ring} \
                                        transition: box-shadow 0.15s;",
                            }
                            span {
                                style: "font-size: 10px; color: {label_color}; font-family: inherit;",
                                "{name}"
                            }
                        }
                    }
                }
            }
        }
    }
}

// ── LangOption ────────────────────────────────────────────────────────────────

/// A selectable language for `LangSwitcher`.
#[derive(Clone, PartialEq)]
pub struct LangOption {
    /// ISO 639-1 language code (e.g. `"en"`).
    pub code: String,
    /// Anglicised name (e.g. `"English"`).
    pub name: String,
    /// Native name (e.g. `"Deutsch"`).
    pub native_name: String,
}

impl LangOption {
    /// Construct a language option.
    pub fn new(
        code: impl Into<String>,
        name: impl Into<String>,
        native_name: impl Into<String>,
    ) -> Self {
        Self {
            code: code.into(),
            name: name.into(),
            native_name: native_name.into(),
        }
    }
}

// ── LangSwitcher ──────────────────────────────────────────────────────────────

#[derive(Props, Clone, PartialEq)]
pub struct LangSwitcherProps {
    /// Available languages.
    pub langs: Vec<LangOption>,
    /// Currently active language code.
    pub active: String,
    /// Called with the newly selected language code.
    #[props(default)]
    pub on_change: EventHandler<String>,
}

/// Select-style dropdown for switching the active language.
///
/// Each option shows both the native name and the anglicised name.
///
/// ```no_run
/// LangSwitcher {
///     langs: vec![LangOption::new("en", "English", "English"), LangOption::new("de", "German", "Deutsch")],
///     active: lang.read().clone(),
///     on_change: move |code| lang.set(code),
/// }
/// ```
#[component]
pub fn LangSwitcher(props: LangSwitcherProps) -> Element {
    rsx! {
        select {
            class: "fs-lang-switcher",
            aria_label: "Select language",
            style: "padding: 6px 10px; border-radius: 6px; font-size: 13px; \
                    background: var(--fs-color-bg-surface, #1e293b); \
                    color: var(--fs-color-text-primary, #e2e8f0); \
                    border: 1px solid var(--fs-color-border-default, #334155); \
                    font-family: inherit; cursor: pointer;",
            onchange: move |e| props.on_change.call(e.value()),

            for lang in &props.langs {
                option {
                    key: "{lang.code}",
                    value: "{lang.code}",
                    selected: lang.code == props.active,
                    "{lang.native_name} ({lang.name})"
                }
            }
        }
    }
}
