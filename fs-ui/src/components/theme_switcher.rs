use dioxus::prelude::*;

/// Props for [`ThemeSwitcher`].
#[derive(Props, Clone, PartialEq)]
pub struct ThemeSwitcherProps {
    /// Available theme names.
    pub themes: Vec<String>,
    /// Currently active theme name.
    pub current_theme: String,
    /// Handler called with the new theme name.
    pub onchange: Option<EventHandler<String>>,
    /// Optional extra CSS classes.
    #[props(default)]
    pub class: String,
}

/// A theme selector that cycles through available themes.
#[component]
pub fn ThemeSwitcher(props: ThemeSwitcherProps) -> Element {
    rsx! {
        div { class: "fs-theme-switcher {props.class}", role: "group", aria_label: "Theme",
            for theme in &props.themes {
                {
                    let theme_name = theme.clone();
                    let is_active = *theme == props.current_theme;
                    let active_class = if is_active { "fs-theme-switcher__btn--active" } else { "" };
                    rsx! {
                        button {
                            class: "fs-theme-switcher__btn {active_class}",
                            aria_pressed: if is_active { "true" } else { "false" },
                            onclick: move |_| {
                                if let Some(h) = &props.onchange {
                                    h.call(theme_name.clone());
                                }
                            },
                            "{theme}"
                        }
                    }
                }
            }
        }
    }
}

#[cfg(feature = "tui")]
mod tui {
    // TUI fallback stub
}
