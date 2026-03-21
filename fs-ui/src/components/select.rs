use dioxus::prelude::*;

/// A single option for [`Select`].
#[derive(Clone, PartialEq)]
pub struct SelectOption {
    /// Machine-readable value.
    pub value: String,
    /// Human-readable label.
    pub label: String,
}

/// Props for [`Select`].
#[derive(Props, Clone, PartialEq)]
pub struct SelectProps {
    /// Currently selected value.
    pub value: String,
    /// Available options.
    pub options: Vec<SelectOption>,
    /// Visible label rendered above the select.
    #[props(default)]
    pub label: String,
    /// Handler called when the selection changes.
    pub onchange: Option<EventHandler<FormEvent>>,
    /// Optional extra CSS classes.
    #[props(default)]
    pub class: String,
}

/// A labelled select/dropdown component.
#[component]
pub fn Select(props: SelectProps) -> Element {
    rsx! {
        div { class: "fs-select-wrapper {props.class}",
            if !props.label.is_empty() {
                label { class: "fs-select__label", "{props.label}" }
            }
            select {
                class: "fs-select",
                value: "{props.value}",
                onchange: move |e| {
                    if let Some(h) = &props.onchange {
                        h.call(e);
                    }
                },
                for opt in &props.options {
                    option {
                        value: "{opt.value}",
                        selected: opt.value == props.value,
                        "{opt.label}"
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
