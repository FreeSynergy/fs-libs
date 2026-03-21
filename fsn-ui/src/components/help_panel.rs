use dioxus::prelude::*;

/// Props for [`HelpPanel`].
#[derive(Props, Clone, PartialEq)]
pub struct HelpPanelProps {
    /// Identifier of the active help topic.
    pub topic_id: String,
    /// Topic title.
    pub title: String,
    /// Main help content (plain text or simple Markdown).
    pub content: String,
    /// Related topic IDs / labels.
    #[props(default)]
    pub related: Vec<String>,
    /// Handler called when the panel is closed.
    pub onclose: Option<EventHandler<MouseEvent>>,
    /// Optional extra CSS classes.
    #[props(default)]
    pub class: String,
}

/// A side-panel that displays context-sensitive help content.
#[component]
pub fn HelpPanel(props: HelpPanelProps) -> Element {
    rsx! {
        aside {
            class: "fsn-help-panel {props.class}",
            aria_label: "Help panel",
            div { class: "fsn-help-panel__header",
                h2 { class: "fsn-help-panel__title", "{props.title}" }
                button {
                    class: "fsn-help-panel__close",
                    aria_label: "Close help panel",
                    onclick: move |e| {
                        if let Some(h) = &props.onclose {
                            h.call(e);
                        }
                    },
                    "✕"
                }
            }
            div { class: "fsn-help-panel__body",
                p { class: "fsn-help-panel__content", "{props.content}" }
            }
            if !props.related.is_empty() {
                div { class: "fsn-help-panel__related",
                    h3 { class: "fsn-help-panel__related-title", "Related topics" }
                    ul { class: "fsn-help-panel__related-list",
                        for topic in &props.related {
                            li { class: "fsn-help-panel__related-item",
                                span { class: "fsn-help-panel__related-link", "{topic}" }
                            }
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
