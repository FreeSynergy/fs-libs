use dioxus::prelude::*;

/// A language option for [`LangSwitcher`].
#[derive(Clone, PartialEq)]
pub struct LangOption {
    /// BCP-47 language code (e.g. `"de"`, `"en"`, `"ar"`).
    pub code: String,
    /// Human-readable language name.
    pub name: String,
    /// Text directionality: `"ltr"` or `"rtl"`.
    pub dir: String,
}

/// Props for [`LangSwitcher`].
#[derive(Props, Clone, PartialEq)]
pub struct LangSwitcherProps {
    /// Available languages.
    pub languages: Vec<LangOption>,
    /// Currently active language code.
    pub current_lang: String,
    /// Handler called with the new language code.
    pub onchange: Option<EventHandler<String>>,
    /// Optional extra CSS classes.
    #[props(default)]
    pub class: String,
}

/// A language selector that respects RTL layout via the `dir` attribute.
#[component]
pub fn LangSwitcher(props: LangSwitcherProps) -> Element {
    // Determine `dir` for the currently active language.
    let current_dir = props
        .languages
        .iter()
        .find(|l| l.code == props.current_lang)
        .map(|l| l.dir.as_str())
        .unwrap_or("ltr")
        .to_string();

    rsx! {
        div {
            class: "fs-lang-switcher {props.class}",
            dir: "{current_dir}",
            role: "group",
            aria_label: "Language",
            for lang in &props.languages {
                {
                    let lang_code = lang.code.clone();
                    let is_active = lang.code == props.current_lang;
                    let active_class = if is_active { "fs-lang-switcher__btn--active" } else { "" };
                    rsx! {
                        button {
                            class: "fs-lang-switcher__btn {active_class}",
                            lang: "{lang.code}",
                            dir: "{lang.dir}",
                            aria_pressed: if is_active { "true" } else { "false" },
                            onclick: move |_| {
                                if let Some(h) = &props.onchange {
                                    h.call(lang_code.clone());
                                }
                            },
                            "{lang.name}"
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
