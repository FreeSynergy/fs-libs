// fs-components/input.rs — Input, Select, Textarea, Checkbox.

use dioxus::prelude::*;

// ── Shared style helpers ──────────────────────────────────────────────────────

fn base_input_style() -> &'static str {
    "width: 100%; box-sizing: border-box; padding: 7px 10px; \
     background: var(--fs-color-bg-surface, #1e293b); \
     color: var(--fs-color-text-primary, #e2e8f0); \
     border: 1px solid var(--fs-color-border-default, #334155); \
     border-radius: 6px; font-size: 13px; font-family: inherit; \
     outline: none; transition: border-color 0.15s;"
}

fn focus_style() -> &'static str {
    "border-color: var(--fs-color-primary, #06b6d4);"
}

// ── Input ─────────────────────────────────────────────────────────────────────

#[derive(Props, Clone, PartialEq)]
pub struct InputProps {
    #[props(default)]
    pub id: String,
    #[props(default)]
    pub name: String,
    #[props(default = "text".to_string())]
    pub r#type: String,
    #[props(default)]
    pub value: String,
    pub placeholder: Option<String>,
    #[props(default = false)]
    pub disabled: bool,
    #[props(default = false)]
    pub required: bool,
    pub aria_label: Option<String>,
    pub aria_describedby: Option<String>,
    #[props(default)]
    pub oninput: EventHandler<FormEvent>,
    #[props(default)]
    pub onchange: EventHandler<FormEvent>,
}

/// Single-line text input with aria support and focus styling.
#[component]
pub fn Input(props: InputProps) -> Element {
    let mut focused = use_signal(|| false);

    let style = format!(
        "{}{}",
        base_input_style(),
        if *focused.read() { focus_style() } else { "" }
    );

    rsx! {
        input {
            id:                  props.id.as_str(),
            name:                props.name.as_str(),
            r#type:              props.r#type.as_str(),
            value:               props.value.as_str(),
            placeholder:         props.placeholder.as_deref().unwrap_or(""),
            disabled:            props.disabled,
            required:            props.required,
            aria_label:          props.aria_label.as_deref().unwrap_or(""),
            aria_describedby:    props.aria_describedby.as_deref().unwrap_or(""),
            style:               "{style}",
            onfocus:  move |_| { *focused.write() = true; },
            onblur:   move |_| { *focused.write() = false; },
            oninput:  move |e| props.oninput.call(e),
            onchange: move |e| props.onchange.call(e),
        }
    }
}

// ── Select ────────────────────────────────────────────────────────────────────

/// A single option in a `Select` element.
#[derive(Clone, PartialEq)]
pub struct SelectOption {
    pub value: String,
    pub label: String,
}

impl SelectOption {
    pub fn new(value: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct SelectProps {
    #[props(default)]
    pub id: String,
    #[props(default)]
    pub name: String,
    pub options: Vec<SelectOption>,
    #[props(default)]
    pub value: String,
    #[props(default = false)]
    pub disabled: bool,
    #[props(default = false)]
    pub required: bool,
    pub aria_label: Option<String>,
    pub aria_describedby: Option<String>,
    #[props(default)]
    pub onchange: EventHandler<FormEvent>,
}

/// Dropdown selector with aria support.
#[component]
pub fn Select(props: SelectProps) -> Element {
    let mut focused = use_signal(|| false);

    let style = format!(
        "{} appearance: none; cursor: pointer;{}",
        base_input_style(),
        if *focused.read() { focus_style() } else { "" }
    );

    rsx! {
        select {
            id:               props.id.as_str(),
            name:             props.name.as_str(),
            disabled:         props.disabled,
            required:         props.required,
            aria_label:       props.aria_label.as_deref().unwrap_or(""),
            aria_describedby: props.aria_describedby.as_deref().unwrap_or(""),
            style:            "{style}",
            onfocus:  move |_| { *focused.write() = true; },
            onblur:   move |_| { *focused.write() = false; },
            onchange: move |e| props.onchange.call(e),

            for opt in &props.options {
                option {
                    value:    opt.value.as_str(),
                    selected: opt.value == props.value,
                    "{opt.label}"
                }
            }
        }
    }
}

// ── Textarea ──────────────────────────────────────────────────────────────────

#[derive(Props, Clone, PartialEq)]
pub struct TextareaProps {
    #[props(default)]
    pub id: String,
    #[props(default)]
    pub name: String,
    #[props(default)]
    pub value: String,
    pub placeholder: Option<String>,
    #[props(default = 4_u32)]
    pub rows: u32,
    #[props(default = false)]
    pub disabled: bool,
    #[props(default = false)]
    pub required: bool,
    pub aria_label: Option<String>,
    pub aria_describedby: Option<String>,
    #[props(default)]
    pub oninput: EventHandler<FormEvent>,
}

/// Multi-line text area with aria support.
#[component]
pub fn Textarea(props: TextareaProps) -> Element {
    let mut focused = use_signal(|| false);

    let style = format!(
        "{} resize: vertical;{}",
        base_input_style(),
        if *focused.read() { focus_style() } else { "" }
    );

    rsx! {
        textarea {
            id:               props.id.as_str(),
            name:             props.name.as_str(),
            rows:             props.rows,
            placeholder:      props.placeholder.as_deref().unwrap_or(""),
            disabled:         props.disabled,
            required:         props.required,
            aria_label:       props.aria_label.as_deref().unwrap_or(""),
            aria_describedby: props.aria_describedby.as_deref().unwrap_or(""),
            style:            "{style}",
            onfocus:  move |_| { *focused.write() = true; },
            onblur:   move |_| { *focused.write() = false; },
            oninput:  move |e| props.oninput.call(e),
            "{props.value}"
        }
    }
}

// ── Checkbox ──────────────────────────────────────────────────────────────────

#[derive(Props, Clone, PartialEq)]
pub struct CheckboxProps {
    #[props(default)]
    pub id: String,
    #[props(default)]
    pub name: String,
    #[props(default = false)]
    pub checked: bool,
    #[props(default = false)]
    pub disabled: bool,
    /// Label shown next to the checkbox.
    pub label: String,
    pub aria_describedby: Option<String>,
    #[props(default)]
    pub onchange: EventHandler<FormEvent>,
}

/// Labeled checkbox with aria support.
#[component]
pub fn Checkbox(props: CheckboxProps) -> Element {
    let cursor = if props.disabled {
        "not-allowed"
    } else {
        "pointer"
    };
    let label_style = format!(
        "display: inline-flex; align-items: center; gap: 8px; cursor: {cursor}; \
         font-size: 13px; color: var(--fs-color-text-primary, #e2e8f0);"
    );
    rsx! {
        label {
            style: "{label_style}",

            input {
                r#type:           "checkbox",
                id:               props.id.as_str(),
                name:             props.name.as_str(),
                checked:          props.checked,
                disabled:         props.disabled,
                aria_describedby: props.aria_describedby.as_deref().unwrap_or(""),
                style:            "width: 15px; height: 15px; cursor: inherit; \
                                   accent-color: var(--fs-color-primary, #06b6d4);",
                onchange: move |e| props.onchange.call(e),
            }
            "{props.label}"
        }
    }
}
