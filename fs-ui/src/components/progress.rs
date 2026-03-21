use dioxus::prelude::*;

/// Props for [`Progress`].
#[derive(Props, Clone, PartialEq)]
pub struct ProgressProps {
    /// Completion ratio in the range `0.0` – `1.0`.
    pub value: f64,
    /// Optional label rendered above the bar.
    pub label: Option<String>,
    /// Optional extra CSS classes.
    #[props(default)]
    pub class: String,
}

/// A horizontal progress bar.
#[component]
pub fn Progress(props: ProgressProps) -> Element {
    let pct = (props.value.clamp(0.0, 1.0) * 100.0) as u32;
    rsx! {
        div { class: "fs-progress-wrapper {props.class}",
            if let Some(lbl) = &props.label {
                span { class: "fs-progress__label", "{lbl}" }
            }
            div {
                class: "fs-progress",
                role: "progressbar",
                aria_valuenow: "{pct}",
                aria_valuemin: "0",
                aria_valuemax: "100",
                div {
                    class: "fs-progress__bar",
                    style: "width: {pct}%; background: var(--fs-color-primary)",
                }
            }
            span { class: "fs-progress__pct", "{pct}%" }
        }
    }
}

#[cfg(feature = "tui")]
mod tui {
    // TUI fallback stub
}
