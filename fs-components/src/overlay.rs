// fs-components/overlay.rs — ContextMenu, NotificationList, HelpPanel, HelpBar.
//
// HelpTopicView / HelpLinkEntry — Dioxus-friendly view models (Clone + PartialEq).
// Built by the caller from a HelpTopic by resolving i18n keys.
//
// HelpPanel  — right-side sliding panel (full help: title, body, links, search).
// HelpBar    — bottom-docked strip (compact field help shown while a field is focused).

use dioxus::prelude::*;

use crate::toast_bus::ToastLevel;

// ── HelpLinkEntry ─────────────────────────────────────────────────────────────

/// A single resolved link entry for the help panel.
///
/// Created by the caller from a `HelpLink` by resolving the i18n label key
/// and picking the icon from `HelpLinkKind::icon()`.
/// Decoupled from `fs-help` so Dioxus props stay `Clone + PartialEq`.
#[derive(Clone, PartialEq)]
pub struct HelpLinkEntry {
    /// Icon character (e.g. `"🌐"`, `"📖"`, `"⑂"`).
    pub icon:  &'static str,
    /// Resolved display label (already translated).
    pub label: String,
    /// Target URL.
    pub url:   String,
}

impl HelpLinkEntry {
    pub fn new(icon: &'static str, label: impl Into<String>, url: impl Into<String>) -> Self {
        Self { icon, label: label.into(), url: url.into() }
    }
}

// ── HelpTopicView ─────────────────────────────────────────────────────────────

/// Resolved view-model for rendering a help topic in Dioxus.
///
/// Created by the caller from a `HelpTopic` (from `fs-help`) by resolving
/// i18n keys. Kept separate so Dioxus props can be `Clone + PartialEq`
/// without needing `Arc<dyn HelpKind>` in prop types.
///
/// # Building a `HelpTopicView`
///
/// ```no_run
/// let view = HelpTopicView::new(
///     fs_i18n::t(topic.title_key()),
///     fs_i18n::t(topic.content_key()),
/// )
/// .with_links(
///     topic.links().iter().map(|l| HelpLinkEntry::new(
///         l.kind.icon(),
///         fs_i18n::t(l.effective_label_key()),
///         l.url.clone(),
///     )).collect()
/// )
/// .with_search_opt(topic.search_query().map(str::to_owned));
/// ```
#[derive(Clone, PartialEq, Default)]
pub struct HelpTopicView {
    pub title:        String,
    pub content:      String,
    pub links:        Vec<HelpLinkEntry>,
    /// Engine-agnostic tutorial search query.
    /// The caller or the `HelpPanel`'s `on_search` handler builds the URL.
    pub search_query: Option<String>,
}

impl HelpTopicView {
    pub fn new(title: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            title:        title.into(),
            content:      content.into(),
            links:        Vec::new(),
            search_query: None,
        }
    }

    pub fn with_links(mut self, links: Vec<HelpLinkEntry>) -> Self {
        self.links = links;
        self
    }

    pub fn with_search(mut self, query: impl Into<String>) -> Self {
        self.search_query = Some(query.into());
        self
    }

    pub fn with_search_opt(mut self, query: Option<String>) -> Self {
        self.search_query = query;
        self
    }

    pub fn is_empty(&self) -> bool {
        self.title.is_empty() && self.content.is_empty()
    }
}

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
    /// External resource links shown below the content (website, docs, git).
    #[props(default)]
    pub links: Vec<HelpLinkEntry>,
    /// Engine-agnostic tutorial search query (emitted via `on_search`).
    #[props(default)]
    pub search_query: Option<String>,
    /// Called when the user closes the panel.
    #[props(default)]
    pub on_close: EventHandler,
    /// Called with a URL when the user clicks a resource link.
    #[props(default)]
    pub on_open_link: EventHandler<String>,
    /// Called with the raw search query when the user clicks "Find tutorials".
    /// The caller should build the search URL using the configured search engine.
    #[props(default)]
    pub on_search: EventHandler<String>,
}

/// Right-side sliding help panel.
///
/// Shows title, body text, optional resource links (website / docs / git),
/// and an optional "Find tutorials" button for external packages.
///
/// ```no_run
/// HelpPanel {
///     title:        topic_view.title.clone(),
///     content:      topic_view.content.clone(),
///     links:        topic_view.links.clone(),
///     search_query: topic_view.search_query.clone(),
///     visible:      *help_open.read(),
///     on_close:     move |_| help_open.set(false),
///     on_open_link: move |url| url_request.set(Some(url)),
///     on_search:    move |q| url_request.set(Some(engine.build_url(&q))),
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

            // ── Header ───────────────────────────────────────────────────────
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

            // ── Scrollable body ───────────────────────────────────────────────
            div {
                style: "flex: 1; overflow-y: auto; padding: 16px; display: flex; flex-direction: column; gap: 16px;",

                // Help text
                pre {
                    style: "margin: 0; white-space: pre-wrap; word-break: break-word; \
                            font-family: inherit; font-size: 13px; line-height: 1.7; \
                            color: var(--fs-color-text-secondary, #94a3b8);",
                    "{props.content}"
                }

                // Resource links
                if !props.links.is_empty() {
                    div {
                        style: "display: flex; flex-direction: column; gap: 6px;",

                        p {
                            style: "margin: 0 0 4px; font-size: 11px; font-weight: 600; \
                                    text-transform: uppercase; letter-spacing: 0.05em; \
                                    color: var(--fs-color-text-muted, #64748b);",
                            "Links"
                        }

                        for link in &props.links {
                            {
                                let url     = link.url.clone();
                                let trigger = props.on_open_link.clone();
                                rsx! {
                                    button {
                                        key: "{link.url}",
                                        style: "display: flex; align-items: center; gap: 8px; \
                                                background: var(--fs-color-bg-elevated, #1e293b); \
                                                border: 1px solid var(--fs-color-border-default, #334155); \
                                                border-radius: 6px; padding: 8px 12px; \
                                                cursor: pointer; text-align: left; width: 100%; \
                                                color: var(--fs-color-primary, #06b6d4); font-size: 13px; \
                                                transition: border-color 150ms;",
                                        onclick: move |_| trigger.call(url.clone()),
                                        span { "{link.icon}" }
                                        span { "{link.label}" }
                                    }
                                }
                            }
                        }
                    }
                }

                // Tutorial search
                if let Some(query) = &props.search_query {
                    {
                        let q       = query.clone();
                        let trigger = props.on_search.clone();
                        rsx! {
                            div {
                                style: "display: flex; flex-direction: column; gap: 6px;",

                                p {
                                    style: "margin: 0 0 4px; font-size: 11px; font-weight: 600; \
                                            text-transform: uppercase; letter-spacing: 0.05em; \
                                            color: var(--fs-color-text-muted, #64748b);",
                                    "Tutorials"
                                }

                                button {
                                    style: "display: flex; align-items: center; gap: 8px; \
                                            background: transparent; \
                                            border: 1px dashed var(--fs-color-border-default, #334155); \
                                            border-radius: 6px; padding: 8px 12px; \
                                            cursor: pointer; text-align: left; width: 100%; \
                                            color: var(--fs-color-text-secondary, #94a3b8); font-size: 13px; \
                                            transition: border-color 150ms;",
                                    onclick: move |_| trigger.call(q.clone()),
                                    span { "🔍" }
                                    span {
                                        style: "overflow: hidden; text-overflow: ellipsis; white-space: nowrap;",
                                        "{q}"
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

// ── HelpBar ───────────────────────────────────────────────────────────────────

#[derive(Props, Clone, PartialEq)]
pub struct HelpBarProps {
    /// Topic to display, or `None` / empty view to hide the bar.
    pub topic: Option<HelpTopicView>,
    /// Called with a URL when the user clicks a resource link.
    #[props(default)]
    pub on_open_link: EventHandler<String>,
    /// Called with the raw search query when the user clicks the search button.
    #[props(default)]
    pub on_search: EventHandler<String>,
}

/// Bottom-docked contextual help bar.
///
/// Place it at the bottom of a form or window. Update the `topic` signal
/// whenever the focused field changes — `HelpSystem::help_for_context(ctx)`
/// returns the right topic using hierarchical fallback.
///
/// Renders nothing when `topic` is `None` or empty.
///
/// ```no_run
/// HelpBar {
///     topic: help_topic.read().clone(),
///     on_open_link: move |url| url_request.set(Some(url)),
///     on_search:    move |q| url_request.set(Some(engine.build_url(&q))),
/// }
/// ```
#[component]
pub fn HelpBar(props: HelpBarProps) -> Element {
    let Some(topic) = &props.topic else { return rsx! { Fragment {} }; };
    if topic.is_empty() { return rsx! { Fragment {} }; }

    rsx! {
        div {
            class: "fs-help-bar",
            style: "display: flex; align-items: center; gap: 12px; \
                    padding: 8px 16px; flex-shrink: 0; \
                    background: var(--fs-color-bg-surface, #1e293b); \
                    border-top: 1px solid var(--fs-color-border-default, #334155); \
                    min-height: 44px;",

            // Help icon + title
            span { style: "font-size: 14px; flex-shrink: 0;", "❓" }
            div { style: "flex: 1; min-width: 0;",
                span {
                    style: "font-size: 12px; font-weight: 600; \
                            color: var(--fs-color-text-primary, #e2e8f0); \
                            margin-right: 6px;",
                    "{topic.title}"
                }
                span {
                    style: "font-size: 12px; color: var(--fs-color-text-muted, #64748b); \
                            overflow: hidden; text-overflow: ellipsis; white-space: nowrap;",
                    "— {topic.content}"
                }
            }

            // Quick-access link buttons
            for link in &topic.links {
                {
                    let url     = link.url.clone();
                    let trigger = props.on_open_link.clone();
                    rsx! {
                        button {
                            key: "{link.url}",
                            title: "{link.label}",
                            style: "background: transparent; border: none; cursor: pointer; \
                                    padding: 4px 6px; border-radius: 4px; font-size: 14px; \
                                    color: var(--fs-color-primary, #06b6d4); \
                                    transition: background 100ms; flex-shrink: 0;",
                            onclick: move |_| trigger.call(url.clone()),
                            "{link.icon}"
                        }
                    }
                }
            }

            // Search button (external packages only)
            if let Some(query) = &topic.search_query {
                {
                    let q       = query.clone();
                    let trigger = props.on_search.clone();
                    rsx! {
                        button {
                            title: "{q}",
                            style: "background: transparent; border: none; cursor: pointer; \
                                    padding: 4px 6px; border-radius: 4px; font-size: 14px; \
                                    color: var(--fs-color-text-muted, #64748b); \
                                    transition: background 100ms; flex-shrink: 0;",
                            onclick: move |_| trigger.call(q.clone()),
                            "🔍"
                        }
                    }
                }
            }
        }
    }
}
