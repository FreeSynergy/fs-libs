use dioxus::prelude::*;

/// Props for [`Modal`].
#[derive(Props, Clone, PartialEq)]
pub struct ModalProps {
    /// Modal title.
    pub title: String,
    /// Whether the modal is currently visible.
    pub visible: bool,
    /// Handler called when the user closes the modal.
    pub onclose: Option<EventHandler<MouseEvent>>,
    /// Optional footer content (e.g. action buttons).
    pub footer: Option<Element>,
    /// Optional extra CSS classes.
    #[props(default)]
    pub class: String,
    /// Modal body content.
    children: Element,
}

/// A modal dialog overlay.
#[component]
pub fn Modal(props: ModalProps) -> Element {
    if !props.visible {
        return rsx! { div { hidden: true } };
    }
    rsx! {
        div {
            class: "fs-modal-backdrop",
            role: "presentation",
            div {
                class: "fs-modal {props.class}",
                role: "dialog",
                aria_modal: "true",
                aria_labelledby: "fs-modal-title",
                div { class: "fs-modal__header",
                    h2 { id: "fs-modal-title", class: "fs-modal__title", "{props.title}" }
                    button {
                        class: "fs-modal__close",
                        aria_label: "Close dialog",
                        onclick: move |e| {
                            if let Some(h) = &props.onclose {
                                h.call(e);
                            }
                        },
                        "✕"
                    }
                }
                div { class: "fs-modal__body", {props.children} }
                if let Some(footer) = props.footer {
                    div { class: "fs-modal__footer", {footer} }
                }
            }
        }
    }
}

#[cfg(feature = "tui")]
mod tui {
    // TUI fallback stub
}
