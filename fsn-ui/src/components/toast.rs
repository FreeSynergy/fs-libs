use dioxus::prelude::*;

/// Severity level for [`Toast`].
#[derive(Clone, PartialEq, Default)]
pub enum ToastLevel {
    #[default]
    Info,
    Success,
    Warning,
    Error,
}

impl ToastLevel {
    fn css_class(&self) -> &'static str {
        match self {
            ToastLevel::Info => "fsn-toast--info",
            ToastLevel::Success => "fsn-toast--success",
            ToastLevel::Warning => "fsn-toast--warning",
            ToastLevel::Error => "fsn-toast--error",
        }
    }

    fn icon(&self) -> &'static str {
        match self {
            ToastLevel::Info => "ℹ",
            ToastLevel::Success => "✓",
            ToastLevel::Warning => "⚠",
            ToastLevel::Error => "✕",
        }
    }
}

/// Props for [`Toast`].
#[derive(Props, Clone, PartialEq)]
pub struct ToastProps {
    /// Notification message text.
    pub message: String,
    /// Severity level.
    #[props(default)]
    pub level: ToastLevel,
    /// Handler called when the user dismisses the toast.
    pub ondismiss: Option<EventHandler<MouseEvent>>,
    /// Optional extra CSS classes.
    #[props(default)]
    pub class: String,
}

/// A transient notification toast.
#[component]
pub fn Toast(props: ToastProps) -> Element {
    let level_class = props.level.css_class();
    let icon = props.level.icon();
    rsx! {
        div {
            class: "fsn-toast {level_class} {props.class}",
            role: "alert",
            aria_live: "polite",
            span { class: "fsn-toast__icon", "{icon}" }
            span { class: "fsn-toast__message", "{props.message}" }
            button {
                class: "fsn-toast__dismiss",
                aria_label: "Dismiss",
                onclick: move |e| {
                    if let Some(h) = &props.ondismiss {
                        h.call(e);
                    }
                },
                "✕"
            }
        }
    }
}

#[cfg(feature = "tui")]
mod tui {
    // TUI fallback stub
}
