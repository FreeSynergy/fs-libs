use dioxus::prelude::*;

/// Props for [`Input`].
#[derive(Props, Clone, PartialEq)]
pub struct InputProps {
    /// Current value of the input.
    pub value: String,
    /// Placeholder text.
    #[props(default)]
    pub placeholder: String,
    /// Visible label rendered above the input.
    #[props(default)]
    pub label: String,
    /// Handler called on every keystroke.
    pub oninput: Option<EventHandler<FormEvent>>,
    /// Disabled state.
    #[props(default = false)]
    pub disabled: bool,
    /// Validation error message shown below the input.
    pub error: Option<String>,
    /// Optional extra CSS classes.
    #[props(default)]
    pub class: String,
}

/// A labelled text input with optional error display.
#[component]
pub fn Input(props: InputProps) -> Element {
    let has_error = props.error.is_some();
    let input_class = if has_error {
        "fs-input fs-input--error"
    } else {
        "fs-input"
    };

    rsx! {
        div { class: "fs-input-wrapper {props.class}",
            if !props.label.is_empty() {
                label { class: "fs-input__label", "{props.label}" }
            }
            input {
                class: "{input_class}",
                value: "{props.value}",
                placeholder: "{props.placeholder}",
                disabled: props.disabled,
                aria_invalid: if has_error { "true" } else { "false" },
                oninput: move |e| {
                    if let Some(h) = &props.oninput {
                        h.call(e);
                    }
                },
            }
            if let Some(err) = &props.error {
                span { class: "fs-input__error", role: "alert", "{err}" }
            }
        }
    }
}

#[cfg(feature = "tui")]
mod tui {
    // TUI fallback stub
}
