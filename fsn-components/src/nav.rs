/// Shared navigation components — tab bar buttons, sidebar nav buttons, and collapsible sidebar.
use dioxus::prelude::*;

/// Horizontal tab-bar button with a bottom-border active indicator.
///
/// Usage:
/// ```rust
/// TabBtn { label: "Browse", is_active: tab == StoreTab::Browse, on_click: move |_| tab.set(StoreTab::Browse) }
/// ```
#[component]
pub fn TabBtn(label: String, is_active: bool, on_click: EventHandler) -> Element {
    let bg     = if is_active { "var(--fsn-color-bg-base)" } else { "transparent" };
    let border = if is_active { "var(--fsn-color-primary)" } else { "transparent" };
    rsx! {
        button {
            style: "padding: 10px 20px; border: none; cursor: pointer; font-size: 14px; \
                    background: {bg}; border-bottom: 2px solid {border};",
            onclick: move |_| on_click.call(()),
            "{label}"
        }
    }
}

// ── FsnSidebar ────────────────────────────────────────────────────────────────

/// A sidebar item descriptor.
///
/// Items with a non-empty `children` list are rendered as folders: clicking them
/// drills into a sub-level instead of calling `on_select`.
#[derive(Clone, PartialEq, Debug)]
pub struct FsnSidebarItem {
    pub id:       String,
    pub icon:     String,
    pub label:    String,
    /// Non-empty = folder item; clicking enters the sub-level.
    pub children: Vec<FsnSidebarItem>,
}

impl FsnSidebarItem {
    /// Create a regular (leaf) sidebar item.
    pub fn new(id: impl Into<String>, icon: impl Into<String>, label: impl Into<String>) -> Self {
        Self { id: id.into(), icon: icon.into(), label: label.into(), children: vec![] }
    }

    /// Create a folder item that drills into `children` when clicked.
    pub fn folder(
        id:       impl Into<String>,
        icon:     impl Into<String>,
        label:    impl Into<String>,
        children: Vec<FsnSidebarItem>,
    ) -> Self {
        Self { id: id.into(), icon: icon.into(), label: label.into(), children }
    }

    pub fn is_folder(&self) -> bool {
        !self.children.is_empty()
    }
}

/// CSS for the collapsible FsnSidebar (icons-only → expands on hover).
/// Inject this once at the app root.
pub const FSN_SIDEBAR_CSS: &str = r#"
.fsn-sidebar {
    width: 48px;
    background: var(--fsn-bg-sidebar, #0a0f1a);
    border-right: 1px solid var(--fsn-border, rgba(148,170,200,0.18));
    display: flex;
    flex-direction: column;
    overflow: hidden;
    transition: width 180ms ease;
    flex-shrink: 0;
    height: 100%;
}
.fsn-sidebar:hover { width: 220px; }
.fsn-sidebar__scroll {
    flex: 1;
    overflow-y: auto;
    overflow-x: hidden;
}
.fsn-sidebar__pinned {
    flex-shrink: 0;
    border-top: 1px solid var(--fsn-border, rgba(148,170,200,0.18));
}
.fsn-sidebar__section-label {
    padding: 12px 14px 4px;
    font-size: 10px;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.08em;
    color: var(--fsn-text-muted, #5a6e88);
    white-space: nowrap;
    overflow: hidden;
    opacity: 0;
    transition: opacity 120ms ease;
}
.fsn-sidebar:hover .fsn-sidebar__section-label { opacity: 1; }
.fsn-sidebar__item {
    display: flex;
    align-items: center;
    gap: 10px;
    width: 100%;
    padding: 9px 14px;
    border: none;
    border-left: 2px solid transparent;
    background: transparent;
    cursor: pointer;
    color: var(--fsn-sidebar-text, #a0b0c8);
    white-space: nowrap;
    overflow: hidden;
    transition: background 120ms, color 120ms, border-color 120ms;
    text-align: left;
    font-size: 13px;
}
.fsn-sidebar__item:hover {
    background: var(--fsn-sidebar-hover-bg, rgba(255,255,255,0.05));
    color: var(--fsn-text-primary, #e8edf5);
}
.fsn-sidebar__item--active {
    background: var(--fsn-sidebar-active-bg, rgba(77,139,245,0.15));
    color: var(--fsn-sidebar-active, #4d8bf5);
    border-left-color: var(--fsn-sidebar-active, #4d8bf5);
}
.fsn-sidebar__icon { font-size: 18px; min-width: 20px; text-align: center; flex-shrink: 0; }
.fsn-sidebar__icon svg { width: 20px; height: 20px; display: block; margin: 0 auto; }
.fsn-sidebar__label { font-size: 13px; overflow: hidden; text-overflow: ellipsis; flex: 1; }
.fsn-sidebar__folder-arrow {
    font-size: 11px;
    color: var(--fsn-text-muted, #5a6e88);
    flex-shrink: 0;
    opacity: 0;
    transition: opacity 120ms ease;
}
.fsn-sidebar:hover .fsn-sidebar__folder-arrow { opacity: 1; }
.fsn-sidebar__back {
    color: var(--fsn-color-primary, #06b6d4) !important;
    font-weight: 600;
}
.fsn-sidebar__divider {
    height: 1px;
    background: var(--fsn-border, rgba(148,170,200,0.18));
    margin: 4px 8px;
    flex-shrink: 0;
}
@keyframes fsn-slide-from-right {
    from { transform: translateX(24px); opacity: 0; }
    to   { transform: translateX(0);    opacity: 1; }
}
@keyframes fsn-slide-from-left {
    from { transform: translateX(-24px); opacity: 0; }
    to   { transform: translateX(0);     opacity: 1; }
}
.fsn-sidebar__level--folder { animation: fsn-slide-from-right 160ms ease; }
.fsn-sidebar__level--root   { animation: fsn-slide-from-left  160ms ease; }
"#;

/// Minimal inline SVG shown when an item has no icon.
const MISSING_ICON_SVG: &str = r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none"><rect x="2" y="2" width="20" height="20" rx="3" stroke="currentColor" stroke-width="1.5" opacity="0.4"/><line x1="6" y1="6" x2="18" y2="18" stroke="#ef4444" stroke-width="2" stroke-linecap="round"/><line x1="18" y1="6" x2="6" y2="18" stroke="#ef4444" stroke-width="2" stroke-linecap="round"/></svg>"##;

/// Renders an icon: inline SVG markup, HTTP(S) URL as <img>, plain emoji/text, or missing-icon placeholder.
#[component]
fn FsnIcon(icon: String) -> Element {
    if icon.trim_start().starts_with("<svg") {
        rsx! { span { class: "fsn-sidebar__icon", dangerous_inner_html: "{icon}" } }
    } else if icon.is_empty() {
        rsx! { span { class: "fsn-sidebar__icon", dangerous_inner_html: MISSING_ICON_SVG } }
    } else if icon.starts_with("http://") || icon.starts_with("https://") || icon.starts_with('/') {
        rsx! {
            span { class: "fsn-sidebar__icon",
                img { src: "{icon}", width: "20", height: "20", style: "object-fit: contain; display: block;" }
            }
        }
    } else {
        rsx! { span { class: "fsn-sidebar__icon", "{icon}" } }
    }
}

/// Resolve the items to actually display for a section.
///
/// Single-item folder rule: if a folder has exactly 1 child, render the child
/// directly instead of the folder itself.
fn resolve_display_items(items: &[FsnSidebarItem]) -> Vec<FsnSidebarItem> {
    items.iter().map(|item| {
        if item.is_folder() && item.children.len() == 1 {
            item.children[0].clone()
        } else {
            item.clone()
        }
    }).collect()
}

/// Renders a list of sidebar items (leaf + folder) using the given signals.
#[component]
fn SidebarItemList(
    items:      Vec<FsnSidebarItem>,
    active_id:  String,
    in_folder:  bool,
    on_select:  EventHandler<String>,
    on_enter:   EventHandler<String>,
) -> Element {
    let display_items = resolve_display_items(&items);
    rsx! {
        for item in display_items {
            if item.is_folder() {
                button {
                    key: "{item.id}",
                    class: "fsn-sidebar__item",
                    title: "{item.label}",
                    onclick: {
                        let fid = item.id.clone();
                        move |_| on_enter.call(fid.clone())
                    },
                    FsnIcon { icon: item.icon.clone() }
                    span { class: "fsn-sidebar__label", "{item.label}" }
                    span { class: "fsn-sidebar__folder-arrow", "›" }
                }
            } else {
                button {
                    key: "{item.id}",
                    class: if item.id == active_id && !in_folder {
                        "fsn-sidebar__item fsn-sidebar__item--active"
                    } else {
                        "fsn-sidebar__item"
                    },
                    title: "{item.label}",
                    onclick: {
                        let id = item.id.clone();
                        move |_| on_select.call(id.clone())
                    },
                    FsnIcon { icon: item.icon.clone() }
                    span { class: "fsn-sidebar__label", "{item.label}" }
                }
            }
        }
    }
}

/// Collapsible sidebar: 48px icons-only, expands to 220px on hover.
/// Supports folder items: clicking a folder drills into its children;
/// a back button at the top returns to the root level.
///
/// `pinned_items` are shown in a separate bottom section with its own independent
/// folder navigation state. Inject `FSN_SIDEBAR_CSS` once at the app root.
#[component]
pub fn FsnSidebar(
    items:     Vec<FsnSidebarItem>,
    #[props(default)]
    pinned_items: Vec<FsnSidebarItem>,
    active_id: String,
    on_select: EventHandler<String>,
) -> Element {
    // Open-folder state for the main (scrollable) section.
    let mut main_open_folder: Signal<Option<String>> = use_signal(|| None);
    // Open-folder state for the pinned section (independent).
    let mut pinned_open_folder: Signal<Option<String>> = use_signal(|| None);

    // ── Main section ──────────────────────────────────────────────────────────
    let main_folder_id = main_open_folder.read().clone();
    let main_in_folder = main_folder_id.is_some();

    let (main_show_items, main_back_label) = if let Some(ref fid) = main_folder_id {
        match items.iter().find(|i| &i.id == fid) {
            Some(folder) => (folder.children.clone(), folder.label.clone()),
            None         => (items.clone(), String::new()),
        }
    } else {
        (items.clone(), String::new())
    };

    let main_level_class = if main_in_folder {
        "fsn-sidebar__level--folder"
    } else {
        "fsn-sidebar__level--root"
    };

    // ── Pinned section ────────────────────────────────────────────────────────
    let pinned_folder_id = pinned_open_folder.read().clone();
    let pinned_in_folder = pinned_folder_id.is_some();

    let (pinned_show_items, pinned_back_label) = if let Some(ref fid) = pinned_folder_id {
        match pinned_items.iter().find(|i| &i.id == fid) {
            Some(folder) => (folder.children.clone(), folder.label.clone()),
            None         => (pinned_items.clone(), String::new()),
        }
    } else {
        (pinned_items.clone(), String::new())
    };

    let pinned_level_class = if pinned_in_folder {
        "fsn-sidebar__level--folder"
    } else {
        "fsn-sidebar__level--root"
    };

    let has_pinned = !pinned_items.is_empty();

    rsx! {
        nav { class: "fsn-sidebar",

            // ── Scrollable main section ───────────────────────────────────────
            div { class: "fsn-sidebar__scroll",
                div { class: "{main_level_class}",
                    if main_in_folder {
                        button {
                            class: "fsn-sidebar__item fsn-sidebar__back",
                            title: "Back",
                            onclick: move |_| main_open_folder.set(None),
                            span { class: "fsn-sidebar__icon", "‹" }
                            span { class: "fsn-sidebar__label", "{main_back_label}" }
                        }
                        div { class: "fsn-sidebar__divider" }
                    }
                    SidebarItemList {
                        items:     main_show_items,
                        active_id: active_id.clone(),
                        in_folder: main_in_folder,
                        on_select: move |id| on_select.call(id),
                        on_enter:  move |fid| main_open_folder.set(Some(fid)),
                    }
                }
            }

            // ── Pinned section (e.g. Settings) ────────────────────────────────
            if has_pinned {
                div { class: "fsn-sidebar__pinned",
                    div { class: "{pinned_level_class}",
                        if pinned_in_folder {
                            button {
                                class: "fsn-sidebar__item fsn-sidebar__back",
                                title: "Back",
                                onclick: move |_| pinned_open_folder.set(None),
                                span { class: "fsn-sidebar__icon", "‹" }
                                span { class: "fsn-sidebar__label", "{pinned_back_label}" }
                            }
                            div { class: "fsn-sidebar__divider" }
                        }
                        SidebarItemList {
                            items:     pinned_show_items,
                            active_id: active_id.clone(),
                            in_folder: pinned_in_folder,
                            on_select: move |id| on_select.call(id),
                            on_enter:  move |fid| pinned_open_folder.set(Some(fid)),
                        }
                    }
                }
            }
        }
    }
}

// ── SidebarNavBtn ─────────────────────────────────────────────────────────────

/// Sidebar navigation button with icon + label and optional left-border active indicator.
///
/// Set `left_border = true` for conductor-style active left border;
/// leave it `false` for settings-style background highlight.
#[component]
pub fn SidebarNavBtn(
    label: String,
    icon: String,
    is_active: bool,
    on_click: EventHandler,
    #[props(default = false)]
    left_border: bool,
) -> Element {
    let bg     = if is_active { "var(--fsn-color-bg-overlay, #1e293b)" } else { "transparent" };
    let color  = if is_active { "var(--fsn-color-primary, #06b6d4)" } else { "var(--fsn-color-text-primary, #e2e8f0)" };
    let weight = if is_active { "600" } else { "400" };
    let border_left = if left_border {
        if is_active { "2px solid var(--fsn-color-primary, #06b6d4)" } else { "2px solid transparent" }
    } else {
        "none"
    };
    rsx! {
        button {
            style: "display: flex; align-items: center; gap: 10px; width: 100%; \
                    padding: 8px 12px; border: none; border-left: {border_left}; \
                    border-radius: var(--fsn-radius-md, 6px); cursor: pointer; \
                    font-size: 14px; text-align: left; background: {bg}; \
                    color: {color}; font-weight: {weight}; margin-bottom: 2px;",
            onclick: move |_| on_click.call(()),
            span { style: "font-size: 16px;", "{icon}" }
            span { "{label}" }
        }
    }
}

// ── FsnTabView ────────────────────────────────────────────────────────────────

/// A tab definition for FsnTabView.
#[derive(Clone, PartialEq, Debug)]
pub struct FsnTabDef {
    pub id:    String,
    pub label: String,
    /// Optional SVG markup or emoji icon shown left of the label.
    pub icon:  Option<String>,
}

impl FsnTabDef {
    pub fn new(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self { id: id.into(), label: label.into(), icon: None }
    }
    pub fn with_icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = Some(icon.into());
        self
    }
}

/// CSS for FsnTabView — inject once at the app root via `style { FSN_TAB_VIEW_CSS }`.
pub const FSN_TAB_VIEW_CSS: &str = r#"
.fsn-tab-view {
    display: flex;
    flex-direction: column;
    height: 100%;
    overflow: hidden;
}
.fsn-tab-view__bar {
    display: flex;
    flex-shrink: 0;
    background: var(--fsn-bg-surface, #162032);
    border-bottom: 1px solid var(--fsn-border, rgba(148,170,200,0.18));
    overflow-x: auto;
    scrollbar-width: none;
}
.fsn-tab-view__bar::-webkit-scrollbar { display: none; }
.fsn-tab-view__tab {
    display: flex;
    align-items: center;
    gap: 7px;
    padding: 10px 20px;
    border: none;
    border-bottom: 2px solid transparent;
    background: transparent;
    color: var(--fsn-text-secondary, #a0b0c8);
    cursor: pointer;
    font-size: 13px;
    font-weight: 500;
    white-space: nowrap;
    transition: color 120ms, border-color 120ms, background 120ms;
}
.fsn-tab-view__tab:hover {
    color: var(--fsn-text-primary, #e8edf5);
    background: var(--fsn-bg-hover, rgba(255,255,255,0.05));
}
.fsn-tab-view__tab--active {
    color: var(--fsn-primary, #4d8bf5);
    border-bottom-color: var(--fsn-primary, #4d8bf5);
}
.fsn-tab-view__tab-icon svg { width: 16px; height: 16px; display: block; }
.fsn-tab-view__content {
    flex: 1;
    overflow: hidden;
}
/* Slide + blur animations — trigger on tab switch */
@keyframes fsn-tab-in-right {
    from { transform: translateX(40px); opacity: 0; filter: blur(6px); }
    to   { transform: translateX(0);    opacity: 1; filter: blur(0);   }
}
@keyframes fsn-tab-in-left {
    from { transform: translateX(-40px); opacity: 0; filter: blur(6px); }
    to   { transform: translateX(0);     opacity: 1; filter: blur(0);   }
}
.fsn-tab-view__content--enter-right {
    animation: fsn-tab-in-right 200ms ease forwards;
}
.fsn-tab-view__content--enter-left {
    animation: fsn-tab-in-left 200ms ease forwards;
}
@media (prefers-reduced-motion: reduce) {
    .fsn-tab-view__content--enter-right,
    .fsn-tab-view__content--enter-left { animation: none; }
}
"#;

/// Universal tab-switching component with slide+blur animation.
///
/// When the user navigates to the right (higher tab index), the new content
/// slides in from the right with a blur-to-sharp effect. Navigating left
/// reverses the direction — exactly like iOS/Android tab switching.
///
/// Works for ANY set of tabs in the application — store sections, settings
/// pages, etc. Inject `FSN_TAB_VIEW_CSS` once at the app root.
///
/// # Example
/// ```rust
/// let tabs = vec![
///     FsnTabDef::new("server", "Server").with_icon(ICON_SVG),
///     FsnTabDef::new("apps",   "Apps"),
///     FsnTabDef::new("desktop","Desktop"),
/// ];
/// FsnTabView { tabs, active_id, on_change: move |id| section.set(id), children }
/// ```
#[component]
pub fn FsnTabView(
    tabs:      Vec<FsnTabDef>,
    active_id: String,
    on_change: EventHandler<String>,
    children:  Element,
) -> Element {
    // Tracks which tab was shown before the current one (for direction detection).
    let mut prev_id:     Signal<String>        = use_signal(|| active_id.clone());
    // Incremented each time the active tab changes — used as the content `key`
    // so that Dioxus remounts the content div and restarts the CSS animation.
    let mut content_key: Signal<u32>           = use_signal(|| 0);
    // The CSS class for the slide animation ("" = no animation on first render).
    let mut anim_class:  Signal<&'static str>  = use_signal(|| "");

    // After each render: if active_id changed, compute direction and trigger remount.
    {
        let new_id  = active_id.clone();
        let t_clone = tabs.clone();
        use_effect(move || {
            let prev = prev_id.peek().clone();
            if prev != new_id {
                let pi  = t_clone.iter().position(|t| t.id == prev).unwrap_or(0);
                let ai  = t_clone.iter().position(|t| t.id == new_id).unwrap_or(0);
                let cls: &'static str = if ai > pi {
                    "fsn-tab-view__content--enter-right"
                } else {
                    "fsn-tab-view__content--enter-left"
                };
                anim_class.set(cls);
                prev_id.set(new_id.clone());
                *content_key.write() += 1;
            }
        });
    }

    let ck  = *content_key.read();
    let cls = *anim_class.read();

    rsx! {
        div { class: "fsn-tab-view",
            // ── Tab bar ───────────────────────────────────────────────────────
            div { class: "fsn-tab-view__bar",
                for tab in &tabs {
                    {
                        let is_active = tab.id == active_id;
                        let tid       = tab.id.clone();
                        rsx! {
                            button {
                                key:   "{tab.id}",
                                class: if is_active {
                                    "fsn-tab-view__tab fsn-tab-view__tab--active"
                                } else {
                                    "fsn-tab-view__tab"
                                },
                                onclick: move |_| on_change.call(tid.clone()),
                                if let Some(icon) = &tab.icon {
                                    if icon.trim_start().starts_with("<svg") {
                                        span { class: "fsn-tab-view__tab-icon",
                                            dangerous_inner_html: "{icon}"
                                        }
                                    } else {
                                        span { class: "fsn-tab-view__tab-icon", "{icon}" }
                                    }
                                }
                                "{tab.label}"
                            }
                        }
                    }
                }
            }
            // ── Content area (keyed so it remounts on tab change) ─────────────
            div {
                key:   "{ck}",
                class: "fsn-tab-view__content {cls}",
                {children}
            }
        }
    }
}
