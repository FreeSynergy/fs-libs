// fs-components/controls.rs — MultiSelect, Toggle, RadioGroup, Slider.

use dioxus::prelude::*;

use crate::input::SelectOption;

// ── MultiSelect ───────────────────────────────────────────────────────────────

#[derive(Props, Clone, PartialEq)]
pub struct MultiSelectProps {
    /// Available options.
    pub options: Vec<SelectOption>,
    /// Currently selected values.
    pub values: Vec<String>,
    /// Called with the updated selection whenever a checkbox changes.
    #[props(default)]
    pub on_change: EventHandler<Vec<String>>,
    /// Max height of the scrollable list (e.g. `"200px"`). Defaults to `"180px"`.
    pub max_height: Option<String>,
}

/// Scrollable multi-option checkbox list.
///
/// Each option renders as a labelled checkbox. `on_change` receives the full
/// updated selection.
///
/// ```no_run
/// MultiSelect {
///     options: vec![SelectOption::new("a", "Alpha"), SelectOption::new("b", "Beta")],
///     values: selected.read().clone(),
///     on_change: move |v| selected.set(v),
/// }
/// ```
#[component]
pub fn MultiSelect(props: MultiSelectProps) -> Element {
    let max_height = props.max_height.as_deref().unwrap_or("180px");

    rsx! {
        div {
            class: "fs-multi-select",
            style: "max-height: {max_height}; overflow-y: auto; \
                    background: var(--fs-color-bg-surface, #1e293b); \
                    border: 1px solid var(--fs-color-border-default, #334155); \
                    border-radius: 6px; padding: 6px 0;",

            for opt in &props.options {
                {
                    let opt_value  = opt.value.clone();
                    let opt_label  = opt.label.clone();
                    let is_checked = props.values.contains(&opt.value);
                    let current    = props.values.clone();
                    let trigger    = props.on_change.clone();

                    rsx! {
                        label {
                            key: "{opt_value}",
                            style: "display: flex; align-items: center; gap: 8px; \
                                    padding: 6px 12px; cursor: pointer; font-size: 13px; \
                                    color: var(--fs-color-text-primary, #e2e8f0); \
                                    transition: background 0.1s;",

                            input {
                                r#type: "checkbox",
                                checked: is_checked,
                                style: "width: 14px; height: 14px; \
                                        accent-color: var(--fs-color-primary, #06b6d4);",
                                onchange: move |_| {
                                    let mut next = current.clone();
                                    if is_checked {
                                        next.retain(|v| v != &opt_value);
                                    } else {
                                        next.push(opt_value.clone());
                                    }
                                    trigger.call(next);
                                },
                            }
                            "{opt_label}"
                        }
                    }
                }
            }
        }
    }
}

// ── Toggle ────────────────────────────────────────────────────────────────────

#[derive(Props, Clone, PartialEq)]
pub struct ToggleProps {
    /// Current on/off state.
    pub checked: bool,
    /// Called with the new state when the toggle is clicked.
    #[props(default)]
    pub on_change: EventHandler<bool>,
    #[props(default = false)]
    pub disabled: bool,
    /// Optional text label rendered to the right of the toggle.
    pub label: Option<String>,
}

/// iOS-style toggle switch with a CSS-animated thumb.
///
/// ```no_run
/// Toggle {
///     checked: *enabled.read(),
///     on_change: move |v| enabled.set(v),
///     label: Some("Enable auto-deploy".to_string()),
/// }
/// ```
#[component]
pub fn Toggle(props: ToggleProps) -> Element {
    let cursor = if props.disabled {
        "not-allowed"
    } else {
        "pointer"
    };

    let track_bg = if props.checked {
        "var(--fs-color-primary, #06b6d4)"
    } else {
        "var(--fs-color-border-default, #334155)"
    };

    let thumb_x = if props.checked { "22px" } else { "2px" };

    rsx! {
        label {
            style: "display: inline-flex; align-items: center; gap: 10px; \
                    cursor: {cursor}; user-select: none;",

            // Hidden native checkbox (accessibility)
            input {
                r#type: "checkbox",
                checked: props.checked,
                disabled: props.disabled,
                style: "position: absolute; opacity: 0; width: 0; height: 0;",
                onchange: move |_| {
                    if !props.disabled {
                        props.on_change.call(!props.checked);
                    }
                },
            }

            // Visual track + thumb
            span {
                aria_hidden: "true",
                style: "position: relative; display: inline-block; \
                        width: 44px; height: 24px; border-radius: 12px; \
                        background: {track_bg}; \
                        transition: background 0.2s; flex-shrink: 0;",

                span {
                    style: "position: absolute; top: 2px; left: {thumb_x}; \
                            width: 20px; height: 20px; border-radius: 50%; \
                            background: #fff; \
                            box-shadow: 0 1px 4px rgba(0,0,0,0.4); \
                            transition: left 0.2s;",
                }
            }

            if let Some(lbl) = &props.label {
                span {
                    style: "font-size: 13px; \
                            color: var(--fs-color-text-primary, #e2e8f0);",
                    "{lbl}"
                }
            }
        }
    }
}

// ── RadioGroup ────────────────────────────────────────────────────────────────

#[derive(Props, Clone, PartialEq)]
pub struct RadioGroupProps {
    /// Options to display.
    pub options: Vec<SelectOption>,
    /// The currently selected value.
    pub value: String,
    /// HTML `name` attribute shared by all radio inputs (required for grouping).
    pub name: String,
    /// Called with the newly selected value.
    #[props(default)]
    pub on_change: EventHandler<String>,
}

/// Vertical group of radio buttons.
///
/// ```no_run
/// RadioGroup {
///     options: vec![SelectOption::new("a", "Option A"), SelectOption::new("b", "Option B")],
///     value: selected.read().clone(),
///     name: "my-group",
///     on_change: move |v| selected.set(v),
/// }
/// ```
#[component]
pub fn RadioGroup(props: RadioGroupProps) -> Element {
    rsx! {
        div {
            class: "fs-radio-group",
            role: "radiogroup",
            style: "display: flex; flex-direction: column; gap: 8px;",

            for opt in &props.options {
                {
                    let opt_value   = opt.value.clone();
                    let opt_label   = opt.label.clone();
                    let is_selected = opt.value == props.value;
                    let trigger     = props.on_change.clone();
                    let name        = props.name.clone();

                    rsx! {
                        label {
                            key: "{opt_value}",
                            style: "display: flex; align-items: center; gap: 8px; \
                                    cursor: pointer; font-size: 13px; \
                                    color: var(--fs-color-text-primary, #e2e8f0);",

                            input {
                                r#type: "radio",
                                name: "{name}",
                                value: "{opt_value}",
                                checked: is_selected,
                                style: "width: 14px; height: 14px; \
                                        accent-color: var(--fs-color-primary, #06b6d4);",
                                onchange: move |_| {
                                    trigger.call(opt_value.clone());
                                },
                            }
                            "{opt_label}"
                        }
                    }
                }
            }
        }
    }
}

// ── Slider ────────────────────────────────────────────────────────────────────

#[derive(Props, Clone, PartialEq)]
pub struct SliderProps {
    /// Minimum value.
    pub min: f64,
    /// Maximum value.
    pub max: f64,
    /// Step increment.
    #[props(default = 1.0_f64)]
    pub step: f64,
    /// Current value.
    pub value: f64,
    /// Called with the new value on input.
    #[props(default)]
    pub on_change: EventHandler<f64>,
    #[props(default = false)]
    pub disabled: bool,
    /// Optional text label shown above the slider.
    pub label: Option<String>,
}

/// Range slider input.
///
/// ```no_run
/// Slider {
///     min: 0.0, max: 100.0, step: 1.0,
///     value: *volume.read(),
///     on_change: move |v| volume.set(v),
///     label: Some("Volume".to_string()),
/// }
/// ```
#[component]
pub fn Slider(props: SliderProps) -> Element {
    rsx! {
        div {
            class: "fs-slider-wrap",
            style: "display: flex; flex-direction: column; gap: 6px;",

            if let Some(lbl) = &props.label {
                div {
                    style: "display: flex; justify-content: space-between; \
                            font-size: 12px; \
                            color: var(--fs-color-text-secondary, #94a3b8);",
                    span { "{lbl}" }
                    span { "{props.value}" }
                }
            }

            input {
                r#type: "range",
                min: "{props.min}",
                max: "{props.max}",
                step: "{props.step}",
                value: "{props.value}",
                disabled: props.disabled,
                style: "width: 100%; accent-color: var(--fs-color-primary, #06b6d4);",
                oninput: move |e| {
                    if let Ok(v) = e.value().parse::<f64>() {
                        props.on_change.call(v);
                    }
                },
            }
        }
    }
}
