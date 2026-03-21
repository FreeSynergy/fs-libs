use dioxus::prelude::*;

/// Props for [`Window`].
#[derive(Props, Clone, PartialEq)]
pub struct WindowProps {
    /// Window title.
    pub title: String,
    /// Whether a close button is shown.
    #[props(default = true)]
    pub closable: bool,
    /// Handler called when the close button is clicked.
    pub onclose: Option<EventHandler<MouseEvent>>,
    /// Optional extra CSS classes.
    #[props(default)]
    pub class: String,
    /// Window content.
    children: Element,
}

/// A floating window panel with a title bar and optional close button.
#[component]
pub fn Window(props: WindowProps) -> Element {
    rsx! {
        div {
            class: "fs-window {props.class}",
            role: "dialog",
            aria_label: "{props.title}",
            div {
                class: "fs-window__titlebar",
                span { class: "fs-window__title", "{props.title}" }
                if props.closable {
                    button {
                        class: "fs-window__close",
                        aria_label: "Close",
                        onclick: move |e| {
                            if let Some(h) = &props.onclose {
                                h.call(e);
                            }
                        },
                        "✕"
                    }
                }
            }
            div { class: "fs-window__body", {props.children} }
        }
    }
}

#[cfg(feature = "tui")]
mod tui {
    // TUI fallback stub
}
