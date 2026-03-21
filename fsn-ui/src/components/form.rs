use dioxus::prelude::*;

/// Props for [`Form`].
#[derive(Props, Clone, PartialEq)]
pub struct FormProps {
    /// Optional form title rendered above the fields.
    pub title: Option<String>,
    /// Handler called on form submission.
    pub onsubmit: Option<EventHandler<FormEvent>>,
    /// Optional extra CSS classes.
    #[props(default)]
    pub class: String,
    /// Form field children.
    children: Element,
}

/// A form container with an optional title.
#[component]
pub fn Form(props: FormProps) -> Element {
    rsx! {
        form {
            class: "fsn-form {props.class}",
            onsubmit: move |e| {
                if let Some(h) = &props.onsubmit {
                    h.call(e);
                }
            },
            if let Some(title) = &props.title {
                h2 { class: "fsn-form__title", "{title}" }
            }
            div { class: "fsn-form__fields", {props.children} }
        }
    }
}

#[cfg(feature = "tui")]
mod tui {
    // TUI fallback stub
}
