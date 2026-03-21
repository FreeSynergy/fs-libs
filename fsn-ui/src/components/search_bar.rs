use dioxus::prelude::*;

/// Props for [`SearchBar`].
#[derive(Props, Clone, PartialEq)]
pub struct SearchBarProps {
    /// Current search string.
    pub value: String,
    /// Placeholder text.
    #[props(default = "Search…".to_string())]
    pub placeholder: String,
    /// Handler called on every keystroke.
    pub oninput: Option<EventHandler<FormEvent>>,
    /// Handler called when the clear button is clicked.
    pub onclear: Option<EventHandler<MouseEvent>>,
    /// Optional extra CSS classes.
    #[props(default)]
    pub class: String,
}

/// A search input with an integrated clear button.
#[component]
pub fn SearchBar(props: SearchBarProps) -> Element {
    let show_clear = !props.value.is_empty();
    rsx! {
        div { class: "fsn-search-bar {props.class}",
            span { class: "fsn-search-bar__icon", aria_hidden: "true", "🔍" }
            input {
                class: "fsn-search-bar__input",
                r#type: "search",
                value: "{props.value}",
                placeholder: "{props.placeholder}",
                aria_label: "Search",
                oninput: move |e| {
                    if let Some(h) = &props.oninput {
                        h.call(e);
                    }
                },
            }
            if show_clear {
                button {
                    class: "fsn-search-bar__clear",
                    aria_label: "Clear search",
                    onclick: move |e| {
                        if let Some(h) = &props.onclear {
                            h.call(e);
                        }
                    },
                    "✕"
                }
            }
        }
    }
}

#[cfg(feature = "tui")]
mod tui {
    // TUI fallback stub
}
