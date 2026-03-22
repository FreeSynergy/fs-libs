// fs-components/overlay.rs — ContextMenu, NotificationList, HelpPanel.

use dioxus::prelude::*;

use crate::toast_bus::ToastLevel;

// ── ContextMenuEntry ──────────────────────────────────────────────────────────

/// A single entry in a `ContextMenu`.
#[derive(Clone, PartialEq)]
pub struct ContextMenuEntry {
    /// Unique key emitted by `on_select`.
    pub key: String,
    /// Display label.
    pub label: String,
    /// Optional icon prefix.
    pub icon: Option<String>,
    /// Disables the item (still visible, not clickable).
    pub disabled: bool,
    /// Renders a horizontal separator line before this entry when `true`.
    pub separator: bool,
}

impl ContextMenuEntry {
    /// Regular enabled entry.
    pub fn new(key: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            label: label.into(),
            icon: None,
            disabled: false,
            separator: false,
        }
    }

    /// Entry with a preceding separator line.
    pub fn with_separator(mut self) -> Self {
        self.separator = true;
        self
    }

    /// Disabled entry.
    pub fn disabled(mut self) -> Self {
        self.disabled = true;
        self
    }
}

// ── ContextMenu ───────────────────────────────────────────────────────────────

#[derive(Props, Clone, PartialEq)]
pub struct ContextMenuProps {
    /// Menu entries.
    pub entries: Vec<ContextMenuEntry>,
    /// Whether the menu is currently visible.
    pub visible: bool,
    /// Horizontal position in viewport pixels.
    pub x: f64,
    /// Vertical position in viewport pixels.
    pub y: f64,
    /// Called with the key of the selected entry.
    #[props(default)]
    pub on_select: EventHandler<String>,
    /// Called when the menu should be closed (e.g. outside click — caller manages this).
    #[props(default)]
    pub on_close: EventHandler,
}

/// Absolute-positioned floating context menu.
///
/// The caller is responsible for positioning (via `x`/`y`) and hiding the menu
/// on outside clicks by listening for global click events.
///
/// ```no_run
/// ContextMenu {
///     entries: menu_entries,
///     visible: *ctx_open.read(),
///     x: ctx_x, y: ctx_y,
///     on_select: move |k| handle_action(k),
///     on_close: move |_| ctx_open.set(false),
/// }
/// ```
#[component]
pub fn ContextMenu(props: ContextMenuProps) -> Element {
    if !props.visible {
        return rsx! { Fragment {} };
    }

    rsx! {
        div {
            class: "fs-context-menu",
            role: "menu",
            style: "position: fixed; top: {props.y}px; left: {props.x}px; z-index: 2000; \
                    min-width: 160px; \
                    background: var(--fs-color-bg-sidebar, #0f172a); \
                    border: 1px solid var(--fs-color-border-default, #334155); \
                    border-radius: 8px; padding: 4px 0; \
                    box-shadow: 0 8px 24px rgba(0,0,0,0.6);",

            for entry in &props.entries {
                {
                    let key      = entry.key.clone();
                    let label    = entry.label.clone();
                    let icon     = entry.icon.clone();
                    let disabled = entry.disabled;
                    let sep      = entry.separator;
                    let trigger  = props.on_select.clone();
                    let close    = props.on_close.clone();

                    let item_style = if disabled {
                        "display: flex; align-items: center; gap: 8px; \
                         padding: 7px 14px; font-size: 13px; cursor: not-allowed; \
                         color: var(--fs-color-text-muted, #64748b); \
                         background: none; border: none; width: 100%; text-align: left; \
                         font-family: inherit;"
                    } else {
                        "display: flex; align-items: center; gap: 8px; \
                         padding: 7px 14px; font-size: 13px; cursor: pointer; \
                         color: var(--fs-color-text-primary, #e2e8f0); \
                         background: none; border: none; width: 100%; text-align: left; \
                         font-family: inherit; transition: background 0.1s;"
                    };

                    rsx! {
                        div { key: "{key}",
                            if sep {
                                div {
                                    style: "height: 1px; margin: 4px 0; \
                                            background: var(--fs-color-border-default, #334155);",
                                }
                            }

                            button {
                                role: "menuitem",
                                disabled: disabled,
                                style: "{item_style}",
                                onclick: move |_| {
                                    if !disabled {
                                        trigger.call(key.clone());
                                        close.call(());
                                    }
                                },

                                if let Some(ic) = &icon {
                                    span { aria_hidden: "true", "{ic}" }
                                }
                                "{label}"
                            }
                        }
                    }
                }
            }
        }
    }
}

// ── NotificationItem ──────────────────────────────────────────────────────────

/// A single notification entry in a `NotificationList`.
#[derive(Clone, PartialEq)]
pub struct NotificationItem {
    /// Unique identifier used for dismiss events.
    pub id: String,
    /// Short title line.
    pub title: String,
    /// Longer description.
    pub message: String,
    /// Severity level (reuses `ToastLevel`).
    pub level: ToastLevel,
    /// Optional human-readable timestamp.
    pub timestamp: Option<String>,
}

impl NotificationItem {
    /// Build a notification item.
    pub fn new(
        id: impl Into<String>,
        title: impl Into<String>,
        message: impl Into<String>,
        level: ToastLevel,
    ) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            message: message.into(),
            level,
            timestamp: None,
        }
    }
}

// ── NotificationList ──────────────────────────────────────────────────────────

#[derive(Props, Clone, PartialEq)]
pub struct NotificationListProps {
    /// Items to display.
    pub items: Vec<NotificationItem>,
    /// Called with the `id` of the dismissed notification.
    #[props(default)]
    pub on_dismiss: EventHandler<String>,
}

/// Stacked list of notification cards with dismiss buttons.
///
/// ```no_run
/// NotificationList {
///     items: notifications.read().clone(),
///     on_dismiss: move |id| notifications.write().retain(|n| n.id != id),
/// }
/// ```
#[component]
pub fn NotificationList(props: NotificationListProps) -> Element {
    rsx! {
        div {
            class: "fs-notification-list",
            style: "display: flex; flex-direction: column; gap: 8px;",

            for item in &props.items {
                {
                    let id      = item.id.clone();
                    let title   = item.title.clone();
                    let message = item.message.clone();
                    let ts      = item.timestamp.clone();
                    let trigger = props.on_dismiss.clone();

                    let accent = item.level.border_css();

                    rsx! {
                        div {
                            key: "{id}",
                            class: "fs-notification-item",
                            style: "display: flex; align-items: flex-start; gap: 10px; \
                                    padding: 12px 14px; border-radius: 8px; \
                                    background: var(--fs-color-bg-surface, #1e293b); \
                                    border: 1px solid var(--fs-color-border-default, #334155); \
                                    {accent}",

                            div { style: "flex: 1; min-width: 0;",
                                p {
                                    style: "margin: 0 0 4px; font-size: 13px; font-weight: 600; \
                                            color: var(--fs-color-text-primary, #e2e8f0);",
                                    "{title}"
                                }
                                p {
                                    style: "margin: 0; font-size: 12px; \
                                            color: var(--fs-color-text-secondary, #94a3b8);",
                                    "{message}"
                                }
                                if let Some(t) = &ts {
                                    p {
                                        style: "margin: 4px 0 0; font-size: 11px; \
                                                color: var(--fs-color-text-muted, #64748b);",
                                        "{t}"
                                    }
                                }
                            }

                            button {
                                aria_label: "Dismiss notification",
                                style: "background: none; border: none; cursor: pointer; \
                                        padding: 0 2px; font-size: 16px; line-height: 1; \
                                        color: var(--fs-color-text-muted, #64748b); \
                                        flex-shrink: 0;",
                                onclick: move |_| trigger.call(id.clone()),
                                "×"
                            }
                        }
                    }
                }
            }
        }
    }
}

// ── HelpPanel ─────────────────────────────────────────────────────────────────

#[derive(Props, Clone, PartialEq)]
pub struct HelpPanelProps {
    /// Panel heading.
    pub title: String,
    /// Help text content (rendered in a `<pre>` block for markdown-like formatting).
    pub content: String,
    /// Whether the panel is visible.
    pub visible: bool,
    /// Called when the user closes the panel.
    #[props(default)]
    pub on_close: EventHandler,
}

/// Right-side sliding help panel.
///
/// Renders as a fixed panel on the right edge of the viewport when `visible` is
/// `true`. Content is shown in a `<pre>` block to preserve markdown-style
/// formatting without a runtime parser.
///
/// ```no_run
/// HelpPanel {
///     title: "Deploying a service",
///     content: help_text,
///     visible: *help_open.read(),
///     on_close: move |_| help_open.set(false),
/// }
/// ```
#[component]
pub fn HelpPanel(props: HelpPanelProps) -> Element {
    if !props.visible {
        return rsx! { Fragment {} };
    }

    rsx! {
        div {
            class: "fs-help-panel",
            role: "complementary",
            aria_label: "Help panel",
            style: "position: fixed; top: 0; right: 0; bottom: 0; z-index: 900; \
                    width: 360px; max-width: 100vw; \
                    display: flex; flex-direction: column; \
                    background: var(--fs-color-bg-sidebar, #0f172a); \
                    border-left: 1px solid var(--fs-color-border-default, #334155); \
                    box-shadow: -8px 0 32px rgba(0,0,0,0.5);",

            // Header
            div {
                style: "display: flex; align-items: center; justify-content: space-between; \
                        padding: 14px 18px; flex-shrink: 0; \
                        border-bottom: 1px solid var(--fs-color-border-default, #334155);",

                span {
                    style: "font-size: 14px; font-weight: 600; \
                            color: var(--fs-color-text-primary, #e2e8f0);",
                    "{props.title}"
                }

                button {
                    aria_label: "Close help panel",
                    style: "background: none; border: none; cursor: pointer; padding: 2px 6px; \
                            font-size: 18px; line-height: 1; \
                            color: var(--fs-color-text-muted, #64748b);",
                    onclick: move |_| props.on_close.call(()),
                    "×"
                }
            }

            // Scrollable content
            div {
                style: "flex: 1; overflow-y: auto; padding: 16px;",

                pre {
                    style: "margin: 0; white-space: pre-wrap; word-break: break-word; \
                            font-family: inherit; font-size: 13px; line-height: 1.7; \
                            color: var(--fs-color-text-secondary, #94a3b8);",
                    "{props.content}"
                }
            }
        }
    }
}
