// fs-components/layout.rs — Tabs, Sidebar, StatusBar, Breadcrumb, SearchBar, ScrollContainer.

use dioxus::prelude::*;

// ── TabItem ───────────────────────────────────────────────────────────────────

/// Single tab descriptor for `Tabs`.
#[derive(Clone, PartialEq)]
pub struct TabItem {
    /// Unique key used for change events.
    pub key: String,
    /// Human-readable tab label.
    pub label: String,
    /// Optional icon (e.g. emoji or short text) shown before the label.
    pub icon: Option<String>,
}

impl TabItem {
    /// Shorthand constructor without an icon.
    pub fn new(key: impl Into<String>, label: impl Into<String>) -> Self {
        Self { key: key.into(), label: label.into(), icon: None }
    }

    /// Attach an icon.
    pub fn with_icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = Some(icon.into());
        self
    }
}

// ── Tabs ──────────────────────────────────────────────────────────────────────

#[derive(Props, Clone, PartialEq)]
pub struct TabsProps {
    /// Tab header items.
    pub items: Vec<TabItem>,
    /// Currently active tab key.
    pub active: String,
    /// Called with the newly selected tab key.
    #[props(default)]
    pub on_change: EventHandler<String>,
    /// Content rendered below the tab bar (typically depends on `active`).
    pub children: Element,
}

/// Full tabbed container with a header row and content area.
///
/// ```no_run
/// Tabs {
///     items: vec![TabItem::new("overview", "Overview"), TabItem::new("logs", "Logs")],
///     active: active_tab.read().clone(),
///     on_change: move |k| active_tab.set(k),
///     match active_tab.read().as_str() {
///         "overview" => rsx! { p { "Overview content" } },
///         _ => rsx! { p { "Logs content" } },
///     }
/// }
/// ```
#[component]
pub fn Tabs(props: TabsProps) -> Element {
    rsx! {
        div {
            class: "fs-tabs",
            style: "display: flex; flex-direction: column;",

            // Tab bar
            div {
                role: "tablist",
                style: "display: flex; gap: 0; \
                        border-bottom: 1px solid var(--fs-color-border-default, #334155);",

                for item in &props.items {
                    {
                        let key      = item.key.clone();
                        let label    = item.label.clone();
                        let icon     = item.icon.clone();
                        let is_active = item.key == props.active;
                        let trigger  = props.on_change.clone();

                        let tab_style = if is_active {
                            "padding: 9px 16px; cursor: pointer; border: none; \
                             background: none; font-size: 13px; font-weight: 600; \
                             font-family: inherit; \
                             color: var(--fs-color-primary, #06b6d4); \
                             border-bottom: 2px solid var(--fs-color-primary, #06b6d4); \
                             margin-bottom: -1px;"
                        } else {
                            "padding: 9px 16px; cursor: pointer; border: none; \
                             background: none; font-size: 13px; font-weight: 400; \
                             font-family: inherit; \
                             color: var(--fs-color-text-secondary, #94a3b8); \
                             border-bottom: 2px solid transparent; \
                             margin-bottom: -1px; \
                             transition: color 0.15s;"
                        };

                        rsx! {
                            button {
                                key: "{key}",
                                role: "tab",
                                aria_selected: if is_active { "true" } else { "false" },
                                style: "{tab_style}",
                                onclick: move |_| trigger.call(key.clone()),

                                if let Some(ic) = &icon {
                                    span {
                                        aria_hidden: "true",
                                        style: "margin-right: 6px;",
                                        "{ic}"
                                    }
                                }
                                "{label}"
                            }
                        }
                    }
                }
            }

            // Content
            div {
                role: "tabpanel",
                style: "padding-top: 16px;",
                {props.children}
            }
        }
    }
}

// ── StatusBar ─────────────────────────────────────────────────────────────────

#[derive(Props, Clone, PartialEq)]
pub struct StatusBarProps {
    /// Content shown on the left side of the bar.
    pub left: Element,
    /// Optional content shown on the right side of the bar.
    pub right: Option<Element>,
}

/// Dark bottom status bar with left and optional right slots.
///
/// ```no_run
/// StatusBar {
///     left: rsx! { span { "Ready" } },
///     right: Some(rsx! { span { "v0.1.0" } }),
/// }
/// ```
#[component]
pub fn StatusBar(props: StatusBarProps) -> Element {
    rsx! {
        div {
            class: "fs-status-bar",
            role: "status",
            style: "display: flex; align-items: center; justify-content: space-between; \
                    padding: 0 12px; height: 24px; flex-shrink: 0; \
                    background: var(--fs-color-bg-sidebar, #0f172a); \
                    border-top: 1px solid var(--fs-color-border-default, #334155); \
                    font-size: 11px; \
                    color: var(--fs-color-text-muted, #64748b);",

            div { style: "display: flex; align-items: center; gap: 12px;",
                {props.left}
            }

            if let Some(right) = props.right {
                div { style: "display: flex; align-items: center; gap: 12px;",
                    {right}
                }
            }
        }
    }
}

// ── BreadcrumbItem ────────────────────────────────────────────────────────────

/// Single breadcrumb step.
#[derive(Clone, PartialEq)]
pub struct BreadcrumbItem {
    /// Display label.
    pub label: String,
    /// Optional navigation href. When `None` the item renders as plain text.
    pub href: Option<String>,
}

impl BreadcrumbItem {
    /// Construct a link step.
    pub fn link(label: impl Into<String>, href: impl Into<String>) -> Self {
        Self { label: label.into(), href: Some(href.into()) }
    }

    /// Construct the current (non-link) step.
    pub fn current(label: impl Into<String>) -> Self {
        Self { label: label.into(), href: None }
    }
}

// ── Breadcrumb ────────────────────────────────────────────────────────────────

#[derive(Props, Clone, PartialEq)]
pub struct BreadcrumbProps {
    /// Ordered list of breadcrumb steps.
    pub items: Vec<BreadcrumbItem>,
}

/// Horizontal breadcrumb trail with `›` separator.
///
/// ```no_run
/// Breadcrumb {
///     items: vec![
///         BreadcrumbItem::link("Home", "/"),
///         BreadcrumbItem::current("Settings"),
///     ],
/// }
/// ```
#[component]
pub fn Breadcrumb(props: BreadcrumbProps) -> Element {
    let last_idx = props.items.len().saturating_sub(1);

    rsx! {
        nav {
            aria_label: "Breadcrumb",
            class: "fs-breadcrumb",
            style: "display: flex; align-items: center; gap: 6px; font-size: 12px;",

            for (idx, item) in props.items.iter().enumerate() {
                {
                    let is_last = idx == last_idx;
                    let label = item.label.clone();
                    let href  = item.href.clone();

                    rsx! {
                        // Separator (not before first item)
                        if idx > 0 {
                            span {
                                aria_hidden: "true",
                                style: "color: var(--fs-color-text-muted, #64748b);",
                                "›"
                            }
                        }

                        if let Some(h) = href {
                            a {
                                href: "{h}",
                                style: "color: var(--fs-color-text-secondary, #94a3b8); \
                                        text-decoration: none;",
                                "{label}"
                            }
                        } else {
                            span {
                                aria_current: if is_last { "page" } else { "" },
                                style: "color: var(--fs-color-text-primary, #e2e8f0);",
                                "{label}"
                            }
                        }
                    }
                }
            }
        }
    }
}

// ── SearchBar ─────────────────────────────────────────────────────────────────

#[derive(Props, Clone, PartialEq)]
pub struct SearchBarProps {
    /// Current input value (controlled).
    pub value: String,
    /// Placeholder text. Defaults to `"Search…"`.
    pub placeholder: Option<String>,
    /// Called on every keystroke.
    #[props(default)]
    pub on_change: EventHandler<String>,
    /// Optional handler called when the user presses Enter.
    pub on_submit: Option<EventHandler<String>>,
}

/// Styled search input with a magnifier icon prefix.
///
/// ```no_run
/// SearchBar {
///     value: query.read().clone(),
///     on_change: move |v| query.set(v),
/// }
/// ```
#[component]
pub fn SearchBar(props: SearchBarProps) -> Element {
    let mut focused = use_signal(|| false);
    let placeholder = props.placeholder.as_deref().unwrap_or("Search…");

    let border_color = if *focused.read() {
        "var(--fs-color-primary, #06b6d4)"
    } else {
        "var(--fs-color-border-default, #334155)"
    };

    rsx! {
        div {
            class: "fs-search-bar",
            style: "display: flex; align-items: center; gap: 8px; \
                    padding: 7px 12px; border-radius: 6px; \
                    background: var(--fs-color-bg-surface, #1e293b); \
                    border: 1px solid {border_color}; \
                    transition: border-color 0.15s;",

            span {
                aria_hidden: "true",
                style: "font-size: 14px; \
                        color: var(--fs-color-text-muted, #64748b); \
                        flex-shrink: 0;",
                "🔍"
            }

            input {
                r#type: "search",
                value: "{props.value}",
                placeholder: "{placeholder}",
                style: "flex: 1; background: none; border: none; outline: none; \
                        font-size: 13px; font-family: inherit; \
                        color: var(--fs-color-text-primary, #e2e8f0);",
                onfocus: move |_| { *focused.write() = true; },
                onblur:  move |_| { *focused.write() = false; },
                oninput: {
                    let trigger = props.on_change.clone();
                    move |e: FormEvent| trigger.call(e.value())
                },
                onkeydown: {
                    let val = props.value.clone();
                    let submit = props.on_submit.clone();
                    move |e: KeyboardEvent| {
                        if e.key() == Key::Enter {
                            if let Some(h) = &submit {
                                h.call(val.clone());
                            }
                        }
                    }
                },
            }
        }
    }
}

// ── ScrollContainer ───────────────────────────────────────────────────────────

#[derive(Props, Clone, PartialEq)]
pub struct ScrollContainerProps {
    /// Content to scroll.
    pub children: Element,
    /// CSS max-height (e.g. `"400px"` or `"60vh"`).
    pub max_height: String,
    /// Additional inline CSS.
    pub extra_style: Option<String>,
}

/// Div with vertical scrolling capped at `max_height`.
///
/// ```no_run
/// ScrollContainer { max_height: "300px",
///     // long list …
/// }
/// ```
#[component]
pub fn ScrollContainer(props: ScrollContainerProps) -> Element {
    let style = format!(
        "max-height: {}; overflow-y: auto; {}",
        props.max_height,
        props.extra_style.as_deref().unwrap_or("")
    );

    rsx! {
        div {
            class: "fs-scroll-container",
            style: "{style}",
            {props.children}
        }
    }
}
