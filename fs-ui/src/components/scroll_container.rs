use dioxus::prelude::*;

/// Props for [`ScrollContainer`].
#[derive(Props, Clone, PartialEq)]
pub struct ScrollContainerProps {
    /// Optional maximum height (e.g. `"400px"`, `"50vh"`).
    /// Defaults to no limit.
    pub max_height: Option<String>,
    /// Optional extra CSS classes.
    #[props(default)]
    pub class: String,
    /// Scrollable content.
    children: Element,
}

/// A vertically scrollable container with an optional maximum height.
#[component]
pub fn ScrollContainer(props: ScrollContainerProps) -> Element {
    let style = match &props.max_height {
        Some(h) => format!("max-height: {h}; overflow-y: auto;"),
        None => "overflow-y: auto;".to_string(),
    };
    rsx! {
        div {
            class: "fs-scroll-container {props.class}",
            style: "{style}",
            {props.children}
        }
    }
}

#[cfg(feature = "tui")]
mod tui {
    // TUI fallback stub
}
