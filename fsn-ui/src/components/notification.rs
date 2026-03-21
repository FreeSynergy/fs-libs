use dioxus::prelude::*;

/// Severity level for [`Notification`].
#[derive(Clone, PartialEq, Default)]
pub enum NotificationLevel {
    #[default]
    Info,
    Success,
    Warning,
    Error,
}

impl NotificationLevel {
    fn css_class(&self) -> &'static str {
        match self {
            NotificationLevel::Info => "fsn-notification--info",
            NotificationLevel::Success => "fsn-notification--success",
            NotificationLevel::Warning => "fsn-notification--warning",
            NotificationLevel::Error => "fsn-notification--error",
        }
    }

    fn icon(&self) -> &'static str {
        match self {
            NotificationLevel::Info => "ℹ",
            NotificationLevel::Success => "✓",
            NotificationLevel::Warning => "⚠",
            NotificationLevel::Error => "✕",
        }
    }
}

/// Props for [`Notification`].
#[derive(Props, Clone, PartialEq)]
pub struct NotificationProps {
    /// Notification title.
    pub title: String,
    /// Notification body text.
    pub message: String,
    /// Severity level.
    #[props(default)]
    pub level: NotificationLevel,
    /// Human-readable timestamp string.
    pub timestamp: String,
    /// Handler called when the user marks the notification as read.
    pub onread: Option<EventHandler<MouseEvent>>,
    /// Optional extra CSS classes.
    #[props(default)]
    pub class: String,
}

/// A persistent notification card (inbox-style).
#[component]
pub fn Notification(props: NotificationProps) -> Element {
    let level_class = props.level.css_class();
    let icon = props.level.icon();
    rsx! {
        article {
            class: "fsn-notification {level_class} {props.class}",
            role: "article",
            div { class: "fsn-notification__header",
                span { class: "fsn-notification__icon", "{icon}" }
                span { class: "fsn-notification__title", "{props.title}" }
                time { class: "fsn-notification__time", "{props.timestamp}" }
            }
            p { class: "fsn-notification__message", "{props.message}" }
            if props.onread.is_some() {
                button {
                    class: "fsn-notification__read-btn",
                    onclick: move |e| {
                        if let Some(h) = &props.onread {
                            h.call(e);
                        }
                    },
                    "Mark as read"
                }
            }
        }
    }
}

#[cfg(feature = "tui")]
mod tui {
    // TUI fallback stub
}
