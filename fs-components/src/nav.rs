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
    let bg     = if is_active { "var(--fs-color-bg-base)" } else { "transparent" };
    let border = if is_active { "var(--fs-color-primary)" } else { "transparent" };
    rsx! {
        button {
            style: "padding: 10px 20px; border: none; cursor: pointer; font-size: 14px; \
                    background: {bg}; border-bottom: 2px solid {border};",
            onclick: move |_| on_click.call(()),
            "{label}"
        }
    }
}

// ── FsSidebar ────────────────────────────────────────────────────────────────

/// A sidebar item descriptor.
///
/// Items with a non-empty `children` list are rendered as folders: clicking them
/// drills into a sub-level instead of calling `on_select`.
#[derive(Clone, PartialEq, Debug)]
pub struct FsSidebarItem {
    pub id:       String,
    pub icon:     String,
    pub label:    String,
    /// Non-empty = folder item; clicking enters the sub-level.
    pub children: Vec<FsSidebarItem>,
}

impl FsSidebarItem {
    /// Create a regular (leaf) sidebar item.
    pub fn new(id: impl Into<String>, icon: impl Into<String>, label: impl Into<String>) -> Self {
        Self { id: id.into(), icon: icon.into(), label: label.into(), children: vec![] }
    }

    /// Create a folder item that drills into `children` when clicked.
    pub fn folder(
        id:       impl Into<String>,
        icon:     impl Into<String>,
        label:    impl Into<String>,
        children: Vec<FsSidebarItem>,
    ) -> Self {
        Self { id: id.into(), icon: icon.into(), label: label.into(), children }
    }

    pub fn is_folder(&self) -> bool {
        !self.children.is_empty()
    }
}

/// CSS for FsSidebar — bookmark-drawer overlay pattern.
/// Inject this once at the app root.
pub const FS_SIDEBAR_CSS: &str = r#"
/* ── Sidebar: bookmark-drawer overlay ───────────────────────────────── */
/* Structure: [panel 220px] [tab-strip compact pill]
   Closed: translateX(-220px) → only the pill visible at the left edge.
   Open  : translateX(0)      → full sidebar + pill visible.
   The pill is vertically centered; above/below it the wallpaper shows. */
.fs-sidebar {
    position: absolute;
    left: 0;
    top: 0;
    bottom: 0;
    z-index: 200;
    display: flex;
    flex-direction: row;
    align-items: center;        /* pill is vertically centered         */
    transform: translateX(-220px);
    transition: transform 220ms cubic-bezier(0.4, 0, 0.2, 1);
    pointer-events: none;       /* transparent above/below the pill    */
}
.fs-sidebar:hover {
    transform: translateX(0);
}
/* ── Panel: the navigation content that slides in ─────────────────── */
.fs-sidebar__panel {
    width: 220px;
    min-width: 220px;
    flex-shrink: 0;
    align-self: stretch;        /* panel fills full height when open   */
    background: var(--fs-bg-sidebar, #0a0f1a);
    border-right: 1px solid var(--fs-border, rgba(148,170,200,0.18));
    display: flex;
    flex-direction: column;
    overflow: hidden;
    pointer-events: all;
    box-shadow: 4px 0 20px rgba(0, 0, 0, 0.5);
}
.fs-sidebar__scroll {
    flex: 1;
    overflow-y: auto;
    overflow-x: hidden;
    display: flex;
    flex-direction: column;
    justify-content: center;
    scrollbar-width: thin;
    scrollbar-color: var(--fs-border) transparent;
}
.fs-sidebar__scroll::-webkit-scrollbar { width: 4px; }
.fs-sidebar__scroll::-webkit-scrollbar-track { background: transparent; }
.fs-sidebar__scroll::-webkit-scrollbar-thumb {
    background: var(--fs-border);
    border-radius: 2px;
}
.fs-sidebar__rail {
    border-top: 1px solid var(--fs-border, rgba(148,170,200,0.18));
    border-bottom: 1px solid var(--fs-border, rgba(148,170,200,0.18));
    padding: 6px 0;
}
.fs-sidebar__pinned {
    flex-shrink: 0;
    border-top: 1px solid var(--fs-border, rgba(148,170,200,0.18));
}
.fs-sidebar__section-label {
    padding: 12px 14px 4px;
    font-size: 10px;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.08em;
    color: var(--fs-text-muted, #5a6e88);
    white-space: nowrap;
    overflow: hidden;
}
.fs-sidebar__item {
    display: flex;
    align-items: center;
    gap: 10px;
    width: 100%;
    padding: 9px 14px;
    border: none;
    border-left: 2px solid transparent;
    background: transparent;
    cursor: pointer;
    color: var(--fs-sidebar-text, #a0b0c8);
    white-space: nowrap;
    overflow: hidden;
    transition: background 120ms, color 120ms, border-color 120ms;
    text-align: left;
    font-size: 13px;
}
.fs-sidebar__item:hover {
    background: var(--fs-sidebar-hover-bg, rgba(255,255,255,0.05));
    color: var(--fs-text-primary, #e8edf5);
}
.fs-sidebar__item--active {
    background: var(--fs-sidebar-active-bg, rgba(77,139,245,0.15));
    color: var(--fs-sidebar-active, #4d8bf5);
    border-left-color: var(--fs-sidebar-active, #4d8bf5);
}
.fs-sidebar__icon { font-size: 18px; min-width: 20px; text-align: center; flex-shrink: 0; }
.fs-sidebar__icon svg { width: 20px; height: 20px; display: block; margin: 0 auto; }
.fs-sidebar__label { font-size: 13px; overflow: hidden; text-overflow: ellipsis; flex: 1; }
.fs-sidebar__folder-arrow {
    font-size: 11px;
    color: var(--fs-text-muted, #5a6e88);
    flex-shrink: 0;
}
.fs-sidebar__back {
    color: var(--fs-color-primary, #06b6d4) !important;
    font-weight: 600;
}
.fs-sidebar__divider {
    height: 1px;
    background: var(--fs-border, rgba(148,170,200,0.18));
    margin: 4px 8px;
    flex-shrink: 0;
}
/* ── Tab strip: compact pill that sticks out from the left edge ─── */
/* Height = auto (only as tall as the icons inside it).
   Vertically centered by the parent align-items: center.
   Rounded right corners give the "Beule" (bump) look.                */
.fs-sidebar__tab-strip {
    width: 44px;
    flex-shrink: 0;
    /* height: auto — sized by content only */
    background: var(--fs-bg-surface, #162032);
    border-top: 1px solid var(--fs-border, rgba(148,170,200,0.18));
    border-right: 1px solid var(--fs-border, rgba(148,170,200,0.18));
    border-bottom: 1px solid var(--fs-border, rgba(148,170,200,0.18));
    border-radius: 0 10px 10px 0;
    display: flex;
    flex-direction: column;
    align-items: center;
    padding: 8px 0;
    gap: 4px;
    pointer-events: all;
    box-shadow: 3px 0 12px rgba(0, 0, 0, 0.35);
}
.fs-sidebar__tab-btn {
    width: 36px;
    height: 36px;
    border: none;
    border-radius: var(--fs-radius-md, 10px);
    background: transparent;
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    color: var(--fs-text-secondary, #a0b0c8);
    transition: background 120ms, color 120ms;
    flex-shrink: 0;
}
.fs-sidebar__tab-btn:hover {
    background: var(--fs-bg-hover, #243352);
    color: var(--fs-text-primary, #e8edf5);
}
.fs-sidebar__tab-btn--active {
    background: var(--fs-sidebar-active-bg, rgba(77,139,245,0.15));
    color: var(--fs-sidebar-active, #4d8bf5);
}
.fs-sidebar__tab-btn svg { width: 20px; height: 20px; display: block; }
/* ── Folder slide animations ─────────────────────────────────────── */
@keyframes fs-slide-from-right {
    from { transform: translateX(24px); opacity: 0; }
    to   { transform: translateX(0);    opacity: 1; }
}
@keyframes fs-slide-from-left {
    from { transform: translateX(-24px); opacity: 0; }
    to   { transform: translateX(0);     opacity: 1; }
}
.fs-sidebar__level--folder { animation: fs-slide-from-right 160ms ease; }
.fs-sidebar__level--root   { animation: fs-slide-from-left  160ms ease; }
/* ── Glass / transparent overrides ──────────────────────────────── */
[data-sidebar-style="glass"] .fs-shell-sidebar,
[data-sidebar-style="glass"] .fs-sidebar__panel {
    background: var(--fs-glass-bg, rgba(22,32,50,0.75)) !important;
    backdrop-filter: blur(var(--fs-glass-blur, 16px));
    -webkit-backdrop-filter: blur(var(--fs-glass-blur, 16px));
}
[data-sidebar-style="transparent"] .fs-shell-sidebar,
[data-sidebar-style="transparent"] .fs-sidebar__panel {
    background: transparent !important;
}
"#;

/// Minimal inline SVG shown when an item has no icon.
const MISSING_ICON_SVG: &str = r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none"><rect x="2" y="2" width="20" height="20" rx="3" stroke="currentColor" stroke-width="1.5" opacity="0.4"/><line x1="6" y1="6" x2="18" y2="18" stroke="#ef4444" stroke-width="2" stroke-linecap="round"/><line x1="18" y1="6" x2="6" y2="18" stroke="#ef4444" stroke-width="2" stroke-linecap="round"/></svg>"##;

/// Renders an icon: inline SVG markup, HTTP(S) URL as <img>, plain emoji/text, or missing-icon placeholder.
#[component]
fn FsIcon(icon: String) -> Element {
    if icon.trim_start().starts_with("<svg") {
        rsx! { span { class: "fs-sidebar__icon", dangerous_inner_html: "{icon}" } }
    } else if icon.is_empty() {
        rsx! { span { class: "fs-sidebar__icon", dangerous_inner_html: MISSING_ICON_SVG } }
    } else if icon.starts_with("http://") || icon.starts_with("https://") || icon.starts_with('/') {
        rsx! {
            span { class: "fs-sidebar__icon",
                img { src: "{icon}", width: "20", height: "20", style: "object-fit: contain; display: block;" }
            }
        }
    } else {
        rsx! { span { class: "fs-sidebar__icon", "{icon}" } }
    }
}

/// Resolve the items to actually display for a section.
///
/// Single-item folder rule: if a folder has exactly 1 child, render the child
/// directly instead of the folder itself.
fn resolve_display_items(items: &[FsSidebarItem]) -> Vec<FsSidebarItem> {
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
    items:      Vec<FsSidebarItem>,
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
                    class: "fs-sidebar__item",
                    title: "{item.label}",
                    onclick: {
                        let fid = item.id.clone();
                        move |_| on_enter.call(fid.clone())
                    },
                    FsIcon { icon: item.icon.clone() }
                    span { class: "fs-sidebar__label", "{item.label}" }
                    span { class: "fs-sidebar__folder-arrow", "›" }
                }
            } else {
                button {
                    key: "{item.id}",
                    class: if item.id == active_id && !in_folder {
                        "fs-sidebar__item fs-sidebar__item--active"
                    } else {
                        "fs-sidebar__item"
                    },
                    title: "{item.label}",
                    onclick: {
                        let id = item.id.clone();
                        move |_| on_select.call(id.clone())
                    },
                    FsIcon { icon: item.icon.clone() }
                    span { class: "fs-sidebar__label", "{item.label}" }
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
/// folder navigation state. Inject `FS_SIDEBAR_CSS` once at the app root.
#[component]
pub fn FsSidebar(
    items:     Vec<FsSidebarItem>,
    #[props(default)]
    pinned_items: Vec<FsSidebarItem>,
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
        "fs-sidebar__level--folder"
    } else {
        "fs-sidebar__level--root"
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
        "fs-sidebar__level--folder"
    } else {
        "fs-sidebar__level--root"
    };

    let has_pinned = !pinned_items.is_empty();

    // Collect root-level icons for the tab-strip (always shows the top-level items).
    let tab_items = resolve_display_items(&items);

    rsx! {
        nav { class: "fs-sidebar",

            // ── Panel: slides in from the left ───────────────────────────────
            div { class: "fs-sidebar__panel",

                // Scrollable main section
                div { class: "fs-sidebar__scroll",
                    div { class: "fs-sidebar__rail",
                        div { class: "{main_level_class}",
                            if main_in_folder {
                                button {
                                    class: "fs-sidebar__item fs-sidebar__back",
                                    title: "Back",
                                    onclick: move |_| main_open_folder.set(None),
                                    span { class: "fs-sidebar__icon", "‹" }
                                    span { class: "fs-sidebar__label", "{main_back_label}" }
                                }
                                div { class: "fs-sidebar__divider" }
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
                }

                // Pinned section (e.g. Settings)
                if has_pinned {
                    div { class: "fs-sidebar__pinned",
                        div { class: "{pinned_level_class}",
                            if pinned_in_folder {
                                button {
                                    class: "fs-sidebar__item fs-sidebar__back",
                                    title: "Back",
                                    onclick: move |_| pinned_open_folder.set(None),
                                    span { class: "fs-sidebar__icon", "‹" }
                                    span { class: "fs-sidebar__label", "{pinned_back_label}" }
                                }
                                div { class: "fs-sidebar__divider" }
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

            // ── Tab strip: bookmark tabs that stick out at the left edge ─────
            // Shows one icon per top-level item. Clicking navigates directly.
            div { class: "fs-sidebar__tab-strip",
                for item in &tab_items {
                    {
                        let is_active_item = item.id == active_id;
                        let tab_class = if is_active_item {
                            "fs-sidebar__tab-btn fs-sidebar__tab-btn--active"
                        } else {
                            "fs-sidebar__tab-btn"
                        };
                        let icon  = item.icon.clone();
                        let label = item.label.clone();
                        let id    = item.id.clone();
                        rsx! {
                            button {
                                key:     "{id}",
                                class:   "{tab_class}",
                                title:   "{label}",
                                onclick: move |_| on_select.call(id.clone()),
                                FsIcon { icon: icon.clone() }
                            }
                        }
                    }
                }
                // Spacer pushes pinned items to the bottom
                div { style: "flex: 1;" }
                for item in &pinned_items {
                    {
                        let is_active_item = item.id == active_id;
                        let tab_class = if is_active_item {
                            "fs-sidebar__tab-btn fs-sidebar__tab-btn--active"
                        } else {
                            "fs-sidebar__tab-btn"
                        };
                        let icon  = item.icon.clone();
                        let label = item.label.clone();
                        let id    = item.id.clone();
                        rsx! {
                            button {
                                key:     "{id}",
                                class:   "{tab_class}",
                                title:   "{label}",
                                onclick: move |_| on_select.call(id.clone()),
                                FsIcon { icon: icon.clone() }
                            }
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
    let bg     = if is_active { "var(--fs-color-bg-overlay, #1e293b)" } else { "transparent" };
    let color  = if is_active { "var(--fs-color-primary, #06b6d4)" } else { "var(--fs-color-text-primary, #e2e8f0)" };
    let weight = if is_active { "600" } else { "400" };
    let border_left = if left_border {
        if is_active { "2px solid var(--fs-color-primary, #06b6d4)" } else { "2px solid transparent" }
    } else {
        "none"
    };
    rsx! {
        button {
            style: "display: flex; align-items: center; gap: 10px; width: 100%; \
                    padding: 8px 12px; border: none; border-left: {border_left}; \
                    border-radius: var(--fs-radius-md, 6px); cursor: pointer; \
                    font-size: 14px; text-align: left; background: {bg}; \
                    color: {color}; font-weight: {weight}; margin-bottom: 2px;",
            onclick: move |_| on_click.call(()),
            span { style: "font-size: 16px;", "{icon}" }
            span { "{label}" }
        }
    }
}

// ── FsTabView ────────────────────────────────────────────────────────────────

/// A tab definition for FsTabView.
#[derive(Clone, PartialEq, Debug)]
pub struct FsTabDef {
    pub id:    String,
    pub label: String,
    /// Optional SVG markup or emoji icon shown left of the label.
    pub icon:  Option<String>,
}

impl FsTabDef {
    pub fn new(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self { id: id.into(), label: label.into(), icon: None }
    }
    pub fn with_icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = Some(icon.into());
        self
    }
}

/// CSS for FsTabView — inject once at the app root via `style { FS_TAB_VIEW_CSS }`.
pub const FS_TAB_VIEW_CSS: &str = r#"
.fs-tab-view {
    display: flex;
    flex-direction: column;
    height: 100%;
    overflow: hidden;
}
.fs-tab-view__bar {
    display: flex;
    justify-content: center;
    flex-shrink: 0;
    background: var(--fs-bg-surface, #162032);
    border-bottom: 1px solid var(--fs-border, rgba(148,170,200,0.18));
    overflow-x: auto;
    scrollbar-width: none;
}
.fs-tab-view__bar::-webkit-scrollbar { display: none; }
.fs-tab-view__tab {
    display: flex;
    align-items: center;
    gap: 7px;
    padding: 10px 20px;
    border: none;
    border-bottom: 2px solid transparent;
    background: transparent;
    color: var(--fs-text-secondary, #a0b0c8);
    cursor: pointer;
    font-size: 13px;
    font-weight: 500;
    white-space: nowrap;
    transition: color 120ms, border-color 120ms, background 120ms;
}
.fs-tab-view__tab:hover {
    color: var(--fs-text-primary, #e8edf5);
    background: var(--fs-bg-hover, rgba(255,255,255,0.05));
}
.fs-tab-view__tab--active {
    color: var(--fs-primary, #4d8bf5);
    border-bottom-color: var(--fs-primary, #4d8bf5);
}
.fs-tab-view__tab-icon svg { width: 16px; height: 16px; display: block; }
.fs-tab-view__content {
    flex: 1;
    overflow: hidden;
}
/* Slide + blur animations — trigger on tab switch */
@keyframes fs-tab-in-right {
    from { transform: translateX(40px); opacity: 0; filter: blur(6px); }
    to   { transform: translateX(0);    opacity: 1; filter: blur(0);   }
}
@keyframes fs-tab-in-left {
    from { transform: translateX(-40px); opacity: 0; filter: blur(6px); }
    to   { transform: translateX(0);     opacity: 1; filter: blur(0);   }
}
.fs-tab-view__content--enter-right {
    animation: fs-tab-in-right 200ms ease forwards;
}
.fs-tab-view__content--enter-left {
    animation: fs-tab-in-left 200ms ease forwards;
}
@media (prefers-reduced-motion: reduce) {
    .fs-tab-view__content--enter-right,
    .fs-tab-view__content--enter-left { animation: none; }
}
"#;

/// Universal tab-switching component with slide+blur animation.
///
/// When the user navigates to the right (higher tab index), the new content
/// slides in from the right with a blur-to-sharp effect. Navigating left
/// reverses the direction — exactly like iOS/Android tab switching.
///
/// Works for ANY set of tabs in the application — store sections, settings
/// pages, etc. Inject `FS_TAB_VIEW_CSS` once at the app root.
///
/// # Example
/// ```rust
/// let tabs = vec![
///     FsTabDef::new("server", "Server").with_icon(ICON_SVG),
///     FsTabDef::new("apps",   "Apps"),
///     FsTabDef::new("desktop","Desktop"),
/// ];
/// FsTabView { tabs, active_id, on_change: move |id| section.set(id), children }
/// ```
#[component]
pub fn FsTabView(
    tabs:      Vec<FsTabDef>,
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
                    "fs-tab-view__content--enter-right"
                } else {
                    "fs-tab-view__content--enter-left"
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
        div { class: "fs-tab-view",
            // ── Tab bar ───────────────────────────────────────────────────────
            div { class: "fs-tab-view__bar",
                for tab in &tabs {
                    {
                        let is_active = tab.id == active_id;
                        let tid       = tab.id.clone();
                        rsx! {
                            button {
                                key:   "{tab.id}",
                                class: if is_active {
                                    "fs-tab-view__tab fs-tab-view__tab--active"
                                } else {
                                    "fs-tab-view__tab"
                                },
                                onclick: move |_| on_change.call(tid.clone()),
                                if let Some(icon) = &tab.icon {
                                    if icon.trim_start().starts_with("<svg") {
                                        span { class: "fs-tab-view__tab-icon",
                                            dangerous_inner_html: "{icon}"
                                        }
                                    } else {
                                        span { class: "fs-tab-view__tab-icon", "{icon}" }
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
                class: "fs-tab-view__content {cls}",
                {children}
            }
        }
    }
}
