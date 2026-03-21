use dioxus::prelude::*;

/// Props for [`StatusBar`].
#[derive(Props, Clone, PartialEq)]
pub struct StatusBarProps {
    /// Text shown on the left side.
    pub left: String,
    /// Text shown on the right side.
    pub right: String,
    /// Optional health indicator symbol (e.g. "✓", "⚠", "✗").
    pub health_indicator: Option<String>,
    /// Optional extra CSS classes.
    #[props(default)]
    pub class: String,
}

/// A horizontal status bar with left/right sections and an optional health badge.
#[component]
pub fn StatusBar(props: StatusBarProps) -> Element {
    rsx! {
        footer {
            class: "fsn-status-bar {props.class}",
            role: "status",
            span { class: "fsn-status-bar__left", "{props.left}" }
            if let Some(indicator) = &props.health_indicator {
                span { class: "fsn-status-bar__health", aria_label: "Health", "{indicator}" }
            }
            span { class: "fsn-status-bar__right", "{props.right}" }
        }
    }
}

#[cfg(feature = "tui")]
mod tui {
    // TUI fallback stub
}
