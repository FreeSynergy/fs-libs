use dioxus::prelude::*;

/// Props for [`IconButton`].
#[derive(Props, Clone, PartialEq)]
pub struct IconButtonProps {
    /// Icon character or emoji (e.g. "⚙").
    pub icon: String,
    /// Tooltip text shown on hover.
    #[props(default)]
    pub tooltip: String,
    /// Disabled state.
    #[props(default = false)]
    pub disabled: bool,
    /// Optional click handler.
    pub onclick: Option<EventHandler<MouseEvent>>,
    /// Optional extra CSS classes.
    #[props(default)]
    pub class: String,
}

/// A round icon-only button with an optional tooltip.
#[component]
pub fn IconButton(props: IconButtonProps) -> Element {
    rsx! {
        button {
            class: "fs-icon-button {props.class}",
            disabled: props.disabled,
            title: "{props.tooltip}",
            aria_label: "{props.tooltip}",
            onclick: move |e| {
                if let Some(h) = &props.onclick {
                    h.call(e);
                }
            },
            span { class: "fs-icon-button__icon", "{props.icon}" }
        }
    }
}

#[cfg(feature = "tui")]
mod tui {
    // TUI fallback stub
}
