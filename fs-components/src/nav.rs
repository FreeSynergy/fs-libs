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

/// Which side of the window the sidebar attaches to.
///
/// - `Left`  → panel slides in from the left; tab-strip appears on the right edge.
/// - `Right` → panel slides in from the right; tab-strip appears on the left edge
///             (mirror layout — only structure is mirrored, not text).
#[derive(Clone, PartialEq, Debug, Default)]
pub enum SidebarSide {
    #[default]
    Left,
    Right,
}

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

/// CSS for FsSidebar — bookmark-drawer overlay pattern, supports left and right side.
///
/// Inject this once at the app root via `style { FS_SIDEBAR_CSS }`.
///
/// **Left sidebar** (default):
///   Panel slides in from the left; parallelogram tab-strip visible at the left edge.
///   Closed: `translateX(-panel_width)` → only the tab-strip visible.
///
/// **Right sidebar** (`side = SidebarSide::Right`):
///   Mirror layout — panel slides in from the right; tab-strip at the right edge.
///   Closed: `translateX(+panel_width)` → only the tab-strip visible.
///   The tab-strip parallelogram is flipped horizontally.
///   Text and icons are NOT mirrored, only the structural layout.
pub const FS_SIDEBAR_CSS: &str = r#"
/* ── Sidebar: bookmark-drawer overlay ──────────────────────────────────
   Structure: [panel] [tab-strip 44px] (left) / [tab-strip 44px] [panel] (right).
   The panel slides behind the tab-strip when collapsed.
   flex-direction: row         → panel on left,  tab-strip on right  (Left)
   flex-direction: row-reverse → panel on right, tab-strip on left   (Right)  */
.fs-sidebar {
    position: absolute;
    top: 0;
    bottom: 0;
    z-index: 200;
    display: flex;
    align-items: stretch;
    transition: transform 220ms cubic-bezier(0.4, 0, 0.2, 1);
    pointer-events: none;
}

/* ── Left variant ────────────────────────────────────────────────────── */
.fs-sidebar--left {
    left: 0;
    flex-direction: row;
    transform: translateX(calc(-1 * var(--fs-sidebar-panel-width, 220px)));
}
.fs-sidebar--left:hover,
.fs-sidebar--left.fs-sidebar--open {
    transform: translateX(0);
}

/* ── Right variant (mirrored) ────────────────────────────────────────── */
.fs-sidebar--right {
    right: 0;
    left: auto;
    flex-direction: row-reverse;
    transform: translateX(var(--fs-sidebar-panel-width, 220px));
}
.fs-sidebar--right:hover,
.fs-sidebar--right.fs-sidebar--open {
    transform: translateX(0);
}

/* ── Panel: the navigation content area ─────────────────────────────── */
.fs-sidebar__panel {
    width: var(--fs-sidebar-panel-width, 220px);
    min-width: var(--fs-sidebar-panel-width, 220px);
    flex-shrink: 0;
    align-self: stretch;
    background: var(--fs-bg-sidebar, #0a0f1a);
    display: flex;
    flex-direction: column;
    overflow: hidden;
    pointer-events: all;
}
.fs-sidebar--left .fs-sidebar__panel {
    border-right: 1px solid var(--fs-border, rgba(148,170,200,0.18));
    box-shadow: 4px 0 20px rgba(0, 0, 0, 0.5);
}
.fs-sidebar--right .fs-sidebar__panel {
    border-left: 1px solid var(--fs-border, rgba(148,170,200,0.18));
    box-shadow: -4px 0 20px rgba(0, 0, 0, 0.5);
}

/* ── Scrollable nav area ─────────────────────────────────────────────── */
.fs-sidebar__scroll {
    flex: 1;
    overflow-y: auto;
    overflow-x: hidden;
    display: flex;
    flex-direction: column;
    justify-content: flex-start;
    scrollbar-width: thin;
    scrollbar-color: var(--fs-border) transparent;
}
.fs-sidebar__scroll::-webkit-scrollbar { width: 4px; }
.fs-sidebar__scroll::-webkit-scrollbar-track { background: transparent; }
.fs-sidebar__scroll::-webkit-scrollbar-thumb {
    background: var(--fs-border);
    border-radius: 2px;
}

/* ── Rail / Pinned sections ──────────────────────────────────────────── */
.fs-sidebar__rail {
    border-top: 1px solid var(--fs-border, rgba(148,170,200,0.18));
    border-bottom: 1px solid var(--fs-border, rgba(148,170,200,0.18));
    padding: 6px 0;
}
.fs-sidebar__pinned {
    flex-shrink: 0;
    border-top: 1px solid var(--fs-border, rgba(148,170,200,0.18));
}

/* ── Section label ───────────────────────────────────────────────────── */
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

/* ── Nav item ────────────────────────────────────────────────────────── */
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
/* Active indicator on the left for left sidebar */
.fs-sidebar--left .fs-sidebar__item--active {
    background: var(--fs-sidebar-active-bg, rgba(77,139,245,0.15));
    color: var(--fs-sidebar-active, #4d8bf5);
    border-left-color: var(--fs-sidebar-active, #4d8bf5);
}
/* Active indicator on the right for right sidebar */
.fs-sidebar--right .fs-sidebar__item {
    border-left: none;
    border-right: 2px solid transparent;
}
.fs-sidebar--right .fs-sidebar__item--active {
    background: var(--fs-sidebar-active-bg, rgba(77,139,245,0.15));
    color: var(--fs-sidebar-active, #4d8bf5);
    border-right-color: var(--fs-sidebar-active, #4d8bf5);
}

/* ── Icon, label, back, folder arrow, divider ────────────────────────── */
.fs-sidebar__icon {
    font-size: 18px;
    min-width: 20px;
    flex-shrink: 0;
    display: flex;
    align-items: center;
    justify-content: center;
}
.fs-sidebar__icon svg { width: 20px; height: 20px; display: block; }
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

/* ── Tab outer: full-height column holding the parallelogram bubbles ─── */
.fs-sidebar__tab-outer {
    align-self: stretch;
    width: 44px;
    min-width: 44px;
    flex-shrink: 0;
    display: flex;
    flex-direction: column;
    pointer-events: none;
    padding: 0;
}

/* ── Tab section: one parallelogram bubble ───────────────────────────── */
/* align-items: center ensures fixed-size buttons are horizontally centered */
.fs-sidebar__tab-section {
    width: 44px;
    flex-shrink: 0;
    background: var(--fs-bg-surface, #162032);
    display: flex;
    flex-direction: column;
    align-items: center;
    padding: 16px 4px;
    gap: 4px;
    pointer-events: all;
}
/* Left sidebar: right-leaning parallelogram (cuts on right side) */
.fs-sidebar--left .fs-sidebar__tab-section {
    clip-path: polygon(0% 0%, 100% 16px, 100% calc(100% - 16px), 0% 100%);
    filter: drop-shadow(3px 0 10px rgba(0,0,0,0.5)) drop-shadow(0 0 1px rgba(148,170,200,0.12));
}
/* Right sidebar: left-leaning parallelogram (mirror — cuts on left side) */
.fs-sidebar--right .fs-sidebar__tab-section {
    clip-path: polygon(0% 16px, 100% 0%, 100% 100%, 0% calc(100% - 16px));
    filter: drop-shadow(-3px 0 10px rgba(0,0,0,0.5)) drop-shadow(0 0 1px rgba(148,170,200,0.12));
}

/* ── Gap between icon section and pinned section ─────────────────────── */
.fs-sidebar__tab-gap {
    flex: 1;
    pointer-events: none;
}

/* ── Tab button: fixed 36×36 square for consistent icon size/alignment ─
   Every icon in every section has identical dimensions, regardless of
   how many items are in the bubble. This eliminates the visual offset
   between the apps bubble (top) and the settings bubble (bottom).       */
.fs-sidebar__tab-btn {
    width: 36px;
    height: 36px;
    flex-shrink: 0;
    border: none;
    border-radius: var(--fs-radius-sm, 6px);
    background: transparent;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 0;
    cursor: pointer;
    color: var(--fs-text-secondary, #a0b0c8);
    transition: background 120ms, color 120ms;
    overflow: hidden;
}
.fs-sidebar__tab-btn:hover {
    background: var(--fs-bg-hover, #243352);
    color: var(--fs-text-primary, #e8edf5);
}
.fs-sidebar__tab-btn--active {
    background: var(--fs-sidebar-active-bg, rgba(77,139,245,0.15));
    color: var(--fs-sidebar-active, #4d8bf5);
}
.fs-sidebar__tab-btn svg { width: 18px; height: 18px; display: block; }
/* Labels hidden in the tab-strip — only icons are shown */
.fs-sidebar__tab-label { display: none; }

/* ── Folder slide animations (diagonal: X + Y offset) ───────────────── */
@keyframes fs-slide-from-right {
    from { transform: translateX(20px) translateY(-6px); opacity: 0; }
    to   { transform: translateX(0)    translateY(0);    opacity: 1; }
}
@keyframes fs-slide-from-left {
    from { transform: translateX(-20px) translateY(6px); opacity: 0; }
    to   { transform: translateX(0)     translateY(0);   opacity: 1; }
}
.fs-sidebar__level--folder { animation: fs-slide-from-right 160ms ease; }
.fs-sidebar__level--root   { animation: fs-slide-from-left  160ms ease; }

/* ── Glass / transparent overrides ──────────────────────────────────── */
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

/// 45° diagonal left-pointing chevron — used for the back button.
const CHEVRON_LEFT: &str = r#"<svg width="14" height="14" viewBox="0 0 14 14" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round"><polyline points="9.5,2 4.5,7 9.5,12"/></svg>"#;

/// 45° diagonal right-pointing chevron — used for folder arrows.
const CHEVRON_RIGHT: &str = r#"<svg width="14" height="14" viewBox="0 0 14 14" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round"><polyline points="4.5,2 9.5,7 4.5,12"/></svg>"#;

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
                    span { class: "fs-sidebar__folder-arrow", dangerous_inner_html: CHEVRON_RIGHT }
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

/// Universal collapsible sidebar — the same component used in every window.
///
/// **Side** controls which edge the sidebar attaches to:
/// - `SidebarSide::Left`  (default): panel on the left, tab-strip on the right.
/// - `SidebarSide::Right`: mirror layout — tab-strip on the left, panel on the right.
///   Only the structure is mirrored; text and icons keep their normal orientation.
///
/// **Panel content** is either the built-in nav list (default) or custom children:
/// - No children → renders `items` as a scrollable nav list in the panel.
/// - With children → renders the children element as the full panel body instead,
///   while `items` are still shown as icons in the tab-strip.
///
/// **panel_width**: panel width in px (default 220). Controls the CSS variable
/// `--fs-sidebar-panel-width` used by both the panel size and the collapse transform.
///
/// **force_open**: when `true`, adds `.fs-sidebar--open` which keeps the sidebar
/// expanded regardless of hover state — use this while drag-resizing.
///
/// Inject `FS_SIDEBAR_CSS` once at the app root.
#[component]
pub fn FsSidebar(
    items:     Vec<FsSidebarItem>,
    #[props(default)]
    pinned_items: Vec<FsSidebarItem>,
    active_id: String,
    on_select: EventHandler<String>,
    #[props(default)]
    side: SidebarSide,
    #[props(default = 220.0_f64)]
    panel_width: f64,
    #[props(default = false)]
    force_open: bool,
    #[props(default)]
    children: Element,
) -> Element {
    // Open-folder state for the main (scrollable) section.
    let mut main_open_folder:   Signal<Option<String>> = use_signal(|| None);
    // Open-folder state for the pinned section (independent).
    let mut pinned_open_folder: Signal<Option<String>> = use_signal(|| None);

    // ── Main section ──────────────────────────────────────────────────────
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

    let main_level_class = if main_in_folder { "fs-sidebar__level--folder" } else { "fs-sidebar__level--root" };

    // ── Pinned section ────────────────────────────────────────────────────
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

    let pinned_level_class = if pinned_in_folder { "fs-sidebar__level--folder" } else { "fs-sidebar__level--root" };

    let has_pinned      = !pinned_items.is_empty();
    let has_custom      = children.is_ok();
    let tab_items       = resolve_display_items(&items);

    let side_class = match side {
        SidebarSide::Left  => "fs-sidebar--left",
        SidebarSide::Right => "fs-sidebar--right",
    };
    let open_class = if force_open { "fs-sidebar--open" } else { "" };

    rsx! {
        nav {
            class: "fs-sidebar {side_class} {open_class}",
            style: "--fs-sidebar-panel-width: {panel_width}px;",

            // ── Panel ────────────────────────────────────────────────────
            // DOM order: panel first, tab-outer second.
            // flex-direction: row         → panel on left  (Left sidebar)
            // flex-direction: row-reverse → panel on right (Right sidebar)
            div { class: "fs-sidebar__panel",

                if has_custom {
                    // Custom content supplied by the caller (e.g. Help panel).
                    {children}
                } else {
                    // Default: scrollable nav list + optional pinned section.

                    div { class: "fs-sidebar__scroll",
                        div { class: "fs-sidebar__rail",
                            div { class: "{main_level_class}",
                                if main_in_folder {
                                    button {
                                        class: "fs-sidebar__item fs-sidebar__back",
                                        title: "Back",
                                        onclick: move |_| main_open_folder.set(None),
                                        span { class: "fs-sidebar__icon", dangerous_inner_html: CHEVRON_LEFT }
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

                    if has_pinned {
                        div { class: "fs-sidebar__pinned",
                            div { class: "{pinned_level_class}",
                                if pinned_in_folder {
                                    button {
                                        class: "fs-sidebar__item fs-sidebar__back",
                                        title: "Back",
                                        onclick: move |_| pinned_open_folder.set(None),
                                        span { class: "fs-sidebar__icon", dangerous_inner_html: CHEVRON_LEFT }
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
            }

            // ── Tab outer: full-height column with parallelogram bubbles ─
            div { class: "fs-sidebar__tab-outer",

                // Icons bubble — one button per top-level item.
                if !tab_items.is_empty() {
                    div { class: "fs-sidebar__tab-section",
                        for item in &tab_items {
                            {
                                let is_active = item.id == active_id;
                                let tab_class = if is_active {
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
                                        span { class: "fs-sidebar__tab-label", "{label}" }
                                    }
                                }
                            }
                        }
                    }
                }

                // Spacer — pushes the pinned bubble to the bottom.
                div { class: "fs-sidebar__tab-gap" }

                // Pinned / settings bubble.
                if has_pinned {
                    div { class: "fs-sidebar__tab-section",
                        for item in &pinned_items {
                            {
                                let is_active = item.id == active_id;
                                let tab_class = if is_active {
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
                                        span { class: "fs-sidebar__tab-label", "{label}" }
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
