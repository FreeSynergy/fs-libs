use dioxus::prelude::*;

/// Visual variant for [`Badge`].
#[derive(Clone, PartialEq, Default)]
pub enum BadgeVariant {
    #[default]
    Default,
    Success,
    Warning,
    Error,
}

impl BadgeVariant {
    fn css_class(&self) -> &'static str {
        match self {
            BadgeVariant::Default => "fsn-badge--default",
            BadgeVariant::Success => "fsn-badge--success",
            BadgeVariant::Warning => "fsn-badge--warning",
            BadgeVariant::Error => "fsn-badge--error",
        }
    }
}

/// Props for [`Badge`].
#[derive(Props, Clone, PartialEq)]
pub struct BadgeProps {
    /// Badge label text.
    pub label: String,
    /// Visual variant.
    #[props(default)]
    pub variant: BadgeVariant,
    /// Optional extra CSS classes.
    #[props(default)]
    pub class: String,
}

/// A small inline badge / pill label.
#[component]
pub fn Badge(props: BadgeProps) -> Element {
    let variant_class = props.variant.css_class();
    rsx! {
        span {
            class: "fsn-badge {variant_class} {props.class}",
            "{props.label}"
        }
    }
}

#[cfg(feature = "tui")]
mod tui {
    // TUI fallback stub
}
