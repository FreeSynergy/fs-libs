// fs-components/form.rs — Form container and FormGrid layout.

use dioxus::prelude::*;

// ── Form ──────────────────────────────────────────────────────────────────────

#[derive(Props, Clone, PartialEq)]
pub struct FormProps {
    /// Form fields and other children.
    pub children: Element,
    /// Submit handler called when the form is submitted.
    #[props(default)]
    pub on_submit: EventHandler<FormEvent>,
    /// Additional inline CSS appended to the form element.
    pub extra_style: Option<String>,
}

/// Semantic form container that calls `on_submit` on submission.
///
/// Prevents default browser form submission behaviour.
///
/// ```no_run
/// Form {
///     on_submit: move |_| { /* handle submit */ },
///     FormField { label: "Name", Input { value: name, oninput: … } }
/// }
/// ```
#[component]
pub fn Form(props: FormProps) -> Element {
    let style = format!(
        "display: flex; flex-direction: column; gap: 14px; {}",
        props.extra_style.as_deref().unwrap_or("")
    );

    rsx! {
        form {
            class: "fs-form",
            style: "{style}",
            onsubmit: move |e| {
                e.prevent_default();
                props.on_submit.call(e);
            },
            {props.children}
        }
    }
}

// ── FormGrid ──────────────────────────────────────────────────────────────────

#[derive(Props, Clone, PartialEq)]
pub struct FormGridProps {
    /// Grid children (typically `FormField` elements).
    pub children: Element,
    /// Number of columns. Defaults to `2`.
    #[props(default = 2_u32)]
    pub cols: u32,
}

/// Responsive CSS grid layout for form fields.
///
/// Lays out children in `cols` equal-width columns (default: 2).
/// Each child spans one cell. On narrow screens the grid collapses to 1 column
/// via `auto-fill` with a minimum cell width of `200px`.
///
/// ```no_run
/// FormGrid {
///     FormField { label: "First name", Input { … } }
///     FormField { label: "Last name",  Input { … } }
/// }
/// ```
#[component]
pub fn FormGrid(props: FormGridProps) -> Element {
    let style = format!(
        "display: grid; \
         grid-template-columns: repeat({}, minmax(200px, 1fr)); \
         gap: 14px;",
        props.cols
    );

    rsx! {
        div {
            class: "fs-form-grid",
            style: "{style}",
            {props.children}
        }
    }
}
