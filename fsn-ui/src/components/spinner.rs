use dioxus::prelude::*;

/// Size variant for [`Spinner`].
#[derive(Clone, PartialEq, Default)]
pub enum SpinnerSize {
    Sm,
    #[default]
    Md,
    Lg,
}

impl SpinnerSize {
    fn css_class(&self) -> &'static str {
        match self {
            SpinnerSize::Sm => "fsn-spinner--sm",
            SpinnerSize::Md => "fsn-spinner--md",
            SpinnerSize::Lg => "fsn-spinner--lg",
        }
    }
}

/// Props for [`Spinner`].
#[derive(Props, Clone, PartialEq)]
pub struct SpinnerProps {
    /// Size variant.
    #[props(default)]
    pub size: SpinnerSize,
    /// Optional accessible loading label.
    pub label: Option<String>,
    /// Optional extra CSS classes.
    #[props(default)]
    pub class: String,
}

/// An animated loading spinner.
#[component]
pub fn Spinner(props: SpinnerProps) -> Element {
    let size_class = props.size.css_class();
    let label = props.label.as_deref().unwrap_or("Loading…");
    rsx! {
        span {
            class: "fsn-spinner {size_class} {props.class}",
            role: "status",
            aria_label: "{label}",
            span { class: "fsn-spinner__ring" }
        }
    }
}

#[cfg(feature = "tui")]
mod tui {
    // TUI fallback stub
}
