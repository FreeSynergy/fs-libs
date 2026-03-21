// fsn-components/form_field.rs — FormField wrapper: Label + Input + Error message.

use dioxus::prelude::*;

// ── FormField ─────────────────────────────────────────────────────────────────

#[derive(Props, Clone, PartialEq)]
pub struct FormFieldProps {
    /// Field label text.
    pub label: String,
    /// `id` of the input inside this field — links the `<label>` via `for`.
    pub field_id: String,
    /// Validation error message. Empty string = no error shown.
    #[props(default)]
    pub error: String,
    /// Hint text shown beneath the input (when no error is displayed).
    pub hint: Option<String>,
    #[props(default = false)]
    pub required: bool,
    /// The input element (or any other control).
    pub children: Element,
}

/// Uniform wrapper for form controls: `<label>` + control + error/hint line.
///
/// The `field_id` is forwarded to the `<label for="…">` attribute so assistive
/// technology correctly associates the label with the control.
///
/// ```no_run
/// FormField {
///     label: "Email".into(),
///     field_id: "email".into(),
///     error: validation_error.clone(),
///     Input { id: "email".into(), r#type: "email".into(), value: email.read().clone(), /* … */ }
/// }
/// ```
#[component]
pub fn FormField(props: FormFieldProps) -> Element {
    let has_error = !props.error.is_empty();
    let hint_id   = format!("{}-hint", props.field_id);
    let error_id  = format!("{}-error", props.field_id);

    rsx! {
        div {
            class: "fsn-form-field",
            style: "display: flex; flex-direction: column; gap: 4px; width: 100%;",

            // ── Label ─────────────────────────────────────────────────────────
            label {
                r#for: props.field_id.as_str(),
                style: "font-size: 12px; font-weight: 600; \
                        color: var(--fsn-color-text-secondary, #94a3b8); \
                        text-transform: uppercase; letter-spacing: 0.04em;",

                "{props.label}"
                if props.required {
                    span {
                        aria_hidden: "true",
                        style: "color: var(--fsn-color-error, #ef4444); margin-left: 3px;",
                        "*"
                    }
                }
            }

            // ── Control (injected by caller) ──────────────────────────────────
            {props.children}

            // ── Error or Hint ─────────────────────────────────────────────────
            if has_error {
                p {
                    id:    error_id.as_str(),
                    role:  "alert",
                    style: "margin: 0; font-size: 12px; \
                            color: var(--fsn-color-error, #ef4444);",
                    "{props.error}"
                }
            } else if let Some(hint) = &props.hint {
                p {
                    id:    hint_id.as_str(),
                    style: "margin: 0; font-size: 12px; \
                            color: var(--fsn-color-text-muted, #64748b);",
                    "{hint}"
                }
            }
        }
    }
}
