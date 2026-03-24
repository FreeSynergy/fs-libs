/// nav.rs — The ONE sidebar for the entire FreeSynergy UI.
///
/// There is exactly ONE `Sidebar` component.  It handles two modes:
///   - `SidebarMode::Overlay`  (default): bookmark-drawer overlay, slides on hover,
///                              shows an icon tab-strip at the edge.  Used by the
///                              desktop shell and window frames.
///   - `SidebarMode::Panel`:   static, always visible, part of the flex layout,
///                              no tab-strip.  Used by inner views (settings sections,
///                              language manager, container app, …).
///
/// Callers supply items; the Sidebar knows nothing about what they represent.
/// Each item can be a leaf or a folder (non-empty children).  Sections add
/// optional labelled groups.  The content is always owned by the caller —
/// the Sidebar is a pure layout/interaction concern.
use dioxus::prelude::*;

// ── TabBtn ────────────────────────────────────────────────────────────────────

/// Horizontal tab-bar button with a bottom-border active indicator.
#[component]
pub fn TabBtn(label: String, is_active: bool, on_click: EventHandler) -> Element {
    let bg = if is_active {
        "var(--fs-color-bg-base)"
    } else {
        "transparent"
    };
    let border = if is_active {
        "var(--fs-color-primary)"
    } else {
        "transparent"
    };
    rsx! {
        button {
            style: "padding: 10px 20px; border: none; cursor: pointer; font-size: 14px; \
                    background: {bg}; border-bottom: 2px solid {border};",
            onclick: move |_| on_click.call(()),
            "{label}"
        }
    }
}

// ── SidebarSide ───────────────────────────────────────────────────────────────

/// Which edge of the window the Sidebar attaches to (Overlay mode only).
///
/// - `Left`  → panel slides in from the left; tab-strip on the right edge.
/// - `Right` → mirror layout — panel on the right; tab-strip on the left edge.
#[derive(Clone, PartialEq, Debug, Default)]
pub enum SidebarSide {
    #[default]
    Left,
    Right,
}

// ── SidebarMode ───────────────────────────────────────────────────────────────

/// Controls how the Sidebar is laid out and behaves.
#[derive(Clone, PartialEq, Debug, Default)]
pub enum SidebarMode {
    /// Bookmark-drawer overlay: `position: absolute`, slides in on hover,
    /// icon tab-strip always visible at the edge.
    /// Used by the desktop shell and per-window sidebars.
    #[default]
    Overlay,
    /// Static panel sidebar: part of the flex layout, always visible,
    /// no sliding, no icon tab-strip.
    /// Used by inner views (settings, language, container, …).
    Panel,
}

// ── SidebarItem ───────────────────────────────────────────────────────────────

/// A single sidebar navigation item.
///
/// Items with non-empty `children` are rendered as folders: clicking them
/// drills into a sub-level instead of calling `on_select`.
#[derive(Clone, PartialEq, Debug)]
pub struct SidebarItem {
    /// Stable identifier — passed to `on_select`.
    pub id: String,
    /// Icon: inline SVG markup, emoji, HTTP(S) URL, or empty string.
    pub icon: String,
    /// Display label.
    pub label: String,
    /// Optional badge (e.g. a count or status indicator).
    pub badge: Option<String>,
    /// Non-empty = folder; clicking drills into the sub-level.
    pub children: Vec<SidebarItem>,
}

impl SidebarItem {
    /// Create a regular (leaf) item.
    pub fn new(id: impl Into<String>, icon: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            icon: icon.into(),
            label: label.into(),
            badge: None,
            children: vec![],
        }
    }

    /// Create a folder item that drills into `children` when clicked.
    pub fn folder(
        id: impl Into<String>,
        icon: impl Into<String>,
        label: impl Into<String>,
        children: Vec<SidebarItem>,
    ) -> Self {
        Self {
            id: id.into(),
            icon: icon.into(),
            label: label.into(),
            badge: None,
            children,
        }
    }

    /// Attach a badge label (builder style).
    pub fn with_badge(mut self, badge: impl Into<String>) -> Self {
        self.badge = Some(badge.into());
        self
    }

    pub fn is_folder(&self) -> bool {
        !self.children.is_empty()
    }
}

// Backward-compatibility alias — callers that still use `FsSidebarItem` compile unchanged.
pub type FsSidebarItem = SidebarItem;

// ── SidebarSection ────────────────────────────────────────────────────────────

/// A labelled group of `SidebarItem`s.
///
/// In Overlay mode, section labels appear inside the scrollable panel.
/// In Panel mode, section labels are shown as uppercase headings.
#[derive(Clone, PartialEq, Debug)]
pub struct SidebarSection {
    /// Optional section heading (shown in both modes when `Some`).
    pub label: Option<String>,
    /// Items in this section.
    pub items: Vec<SidebarItem>,
}

impl SidebarSection {
    /// Construct a section with a heading.
    pub fn new(label: impl Into<String>, items: Vec<SidebarItem>) -> Self {
        Self {
            label: Some(label.into()),
            items,
        }
    }

    /// Construct an untitled section (no heading).
    pub fn untitled(items: Vec<SidebarItem>) -> Self {
        Self { label: None, items }
    }
}

// ── CSS ───────────────────────────────────────────────────────────────────────

/// CSS for `Sidebar` — inject once at the app root via `style { FS_SIDEBAR_CSS }`.
///
/// Covers both Overlay (bookmark-drawer with icon tab-strip) and Panel (static)
/// rendering modes.
pub const FS_SIDEBAR_CSS: &str = r#"
/* ── Overlay sidebar: bookmark-drawer ──────────────────────────────────
   Structure: [panel] [tab-strip 44px]  (Left)
            / [tab-strip 44px] [panel]  (Right)
   The panel slides behind the tab-strip when collapsed.              */
.fs-sidebar {
    position: absolute;
    top: 0;
    bottom: 0;
    z-index: 200;
    display: flex;
    align-items: stretch;
    transition: transform 220ms cubic-bezier(0.4, 0, 0.2, 1);
}

/* ── Left variant ─────────────────────────────────────────────────── */
.fs-sidebar--left {
    left: 0;
    flex-direction: row;
    transform: translateX(calc(-1 * var(--fs-sidebar-panel-width, 220px)));
}
.fs-sidebar--left:hover,
.fs-sidebar--left.fs-sidebar--open {
    transform: translateX(0);
}

/* ── Right variant ────────────────────────────────────────────────── */
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

/* ── Panel mode: static, always visible, no tab-strip ─────────────── */
.fs-sidebar--panel {
    position: relative;
    display: flex;
    flex-direction: column;
    width: var(--fs-sidebar-panel-width, 220px);
    min-width: var(--fs-sidebar-panel-width, 220px);
    flex-shrink: 0;
    background: var(--fs-bg-sidebar, #0a0f1a);
    border-right: 1px solid var(--fs-border, rgba(148,170,200,0.18));
    overflow: hidden;
    /* Override the overlay positioning */
    transform: none !important;
    top: auto;
    bottom: auto;
    z-index: auto;
}
.fs-sidebar--panel .fs-sidebar__scroll {
    flex: 1;
}
.fs-sidebar--panel .fs-sidebar__section-label {
    padding: 10px 14px 4px;
}

/* ── Panel (the sliding content area in Overlay mode) ─────────────── */
.fs-sidebar__panel {
    width: var(--fs-sidebar-panel-width, 220px);
    min-width: var(--fs-sidebar-panel-width, 220px);
    flex-shrink: 0;
    align-self: stretch;
    background: var(--fs-bg-sidebar, #0a0f1a);
    display: flex;
    flex-direction: column;
    overflow: hidden;
}
.fs-sidebar--left .fs-sidebar__panel {
    border-right: 1px solid var(--fs-border, rgba(148,170,200,0.18));
    box-shadow: 4px 0 20px rgba(0, 0, 0, 0.5);
}
.fs-sidebar--right .fs-sidebar__panel {
    border-left: 1px solid var(--fs-border, rgba(148,170,200,0.18));
    box-shadow: -4px 0 20px rgba(0, 0, 0, 0.5);
}

/* ── Scrollable nav area ──────────────────────────────────────────── */
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

/* ── Rail / Pinned sections ───────────────────────────────────────── */
.fs-sidebar__rail {
    border-top: 1px solid var(--fs-border, rgba(148,170,200,0.18));
    border-bottom: 1px solid var(--fs-border, rgba(148,170,200,0.18));
    padding: 6px 0;
}
.fs-sidebar__pinned {
    flex-shrink: 0;
    border-top: 1px solid var(--fs-border, rgba(148,170,200,0.18));
}

/* ── Section label ────────────────────────────────────────────────── */
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

/* ── Nav item ─────────────────────────────────────────────────────── */
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
    font-family: inherit;
}
.fs-sidebar__item:hover {
    background: var(--fs-sidebar-hover-bg, rgba(255,255,255,0.05));
    color: var(--fs-text-primary, #e8edf5);
}
.fs-sidebar--left .fs-sidebar__item--active,
.fs-sidebar--panel .fs-sidebar__item--active {
    background: var(--fs-sidebar-active-bg, rgba(77,139,245,0.15));
    color: var(--fs-sidebar-active, #4d8bf5);
    border-left-color: var(--fs-sidebar-active, #4d8bf5);
}
.fs-sidebar--right .fs-sidebar__item {
    border-left: none;
    border-right: 2px solid transparent;
}
.fs-sidebar--right .fs-sidebar__item--active {
    background: var(--fs-sidebar-active-bg, rgba(77,139,245,0.15));
    color: var(--fs-sidebar-active, #4d8bf5);
    border-right-color: var(--fs-sidebar-active, #4d8bf5);
}

/* ── Badge ────────────────────────────────────────────────────────── */
.fs-sidebar__badge {
    margin-left: auto;
    font-size: 10px;
    font-weight: 700;
    padding: 1px 6px;
    border-radius: 999px;
    background: var(--fs-color-primary, #06b6d4);
    color: #000;
    flex-shrink: 0;
}

/* ── Icon, label, back, folder arrow, divider ─────────────────────── */
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

/* ── Tab outer: full-height column holding the pill bubbles ────────── */
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

/* ── Tab section: one pill bubble ──────────────────────────────────── */
.fs-sidebar__tab-section {
    width: 44px;
    flex-shrink: 0;
    background: var(--fs-bg-surface, #162032);
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 8px 4px;
    gap: 4px;
    pointer-events: all;
}
.fs-sidebar--left .fs-sidebar__tab-section {
    border-radius: 0 12px 12px 0;
    box-shadow: 3px 0 10px rgba(0,0,0,0.5), 0 0 1px rgba(148,170,200,0.12);
}
.fs-sidebar--right .fs-sidebar__tab-section {
    border-radius: 12px 0 0 12px;
    box-shadow: -3px 0 10px rgba(0,0,0,0.5), 0 0 1px rgba(148,170,200,0.12);
}

/* ── Gap between icon section and pinned section ─────────────────── */
.fs-sidebar__tab-gap {
    flex: 1;
    pointer-events: none;
}

/* ── Tab button ────────────────────────────────────────────────────── */
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
.fs-sidebar__tab-label { display: none; }

/* ── Folder slide animations ────────────────────────────────────────── */
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

/* ── Glass / transparent overrides ─────────────────────────────────── */
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

// ── Icon renderer ─────────────────────────────────────────────────────────────

const MISSING_ICON_SVG: &str = r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none"><rect x="2" y="2" width="20" height="20" rx="3" stroke="currentColor" stroke-width="1.5" opacity="0.4"/><line x1="6" y1="6" x2="18" y2="18" stroke="#ef4444" stroke-width="2" stroke-linecap="round"/><line x1="18" y1="6" x2="6" y2="18" stroke="#ef4444" stroke-width="2" stroke-linecap="round"/></svg>"##;
const CHEVRON_LEFT: &str = r#"<svg width="14" height="14" viewBox="0 0 14 14" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round"><polyline points="9.5,2 4.5,7 9.5,12"/></svg>"#;
const CHEVRON_RIGHT: &str = r#"<svg width="14" height="14" viewBox="0 0 14 14" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round"><polyline points="4.5,2 9.5,7 4.5,12"/></svg>"#;

/// Renders an icon: inline SVG markup, URL as `<img>`, plain emoji/text, or placeholder.
#[component]
fn SidebarIcon(icon: String) -> Element {
    if icon.trim_start().starts_with("<svg") {
        rsx! { span { class: "fs-sidebar__icon", dangerous_inner_html: "{icon}" } }
    } else if icon.is_empty() {
        rsx! { span { class: "fs-sidebar__icon", dangerous_inner_html: MISSING_ICON_SVG } }
    } else if icon.starts_with("http://") || icon.starts_with("https://") || icon.starts_with('/') {
        rsx! {
            span { class: "fs-sidebar__icon",
                img { src: "{icon}", width: "20", height: "20",
                      style: "object-fit: contain; display: block;" }
            }
        }
    } else {
        rsx! { span { class: "fs-sidebar__icon", "{icon}" } }
    }
}

// ── Item list helper ──────────────────────────────────────────────────────────

/// Single-item folder rule: if a folder has exactly one child, render the child directly.
fn resolve_display_items(items: &[SidebarItem]) -> Vec<SidebarItem> {
    items
        .iter()
        .map(|item| {
            if item.is_folder() && item.children.len() == 1 {
                item.children[0].clone()
            } else {
                item.clone()
            }
        })
        .collect()
}

/// Renders a flat list of sidebar items (leaf + folder) for one level.
#[component]
fn SidebarItemList(
    items: Vec<SidebarItem>,
    active_id: String,
    in_folder: bool,
    on_select: EventHandler<String>,
    on_enter: EventHandler<String>,
    #[props(default)] on_context_menu: Option<EventHandler<String>>,
) -> Element {
    let display = resolve_display_items(&items);
    rsx! {
        for item in display {
            if item.is_folder() {
                button {
                    key:   "{item.id}",
                    class: "fs-sidebar__item",
                    title: "{item.label}",
                    onclick: {
                        let fid = item.id.clone();
                        move |_| on_enter.call(fid.clone())
                    },
                    SidebarIcon { icon: item.icon.clone() }
                    span { class: "fs-sidebar__label", "{item.label}" }
                    if let Some(b) = &item.badge {
                        span { class: "fs-sidebar__badge", "{b}" }
                    }
                    span { class: "fs-sidebar__folder-arrow",
                           dangerous_inner_html: CHEVRON_RIGHT }
                }
            } else {
                button {
                    key:   "{item.id}",
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
                    oncontextmenu: {
                        let id = item.id.clone();
                        move |evt: MouseEvent| {
                            evt.prevent_default();
                            if let Some(ref h) = on_context_menu {
                                h.call(id.clone());
                            }
                        }
                    },
                    SidebarIcon { icon: item.icon.clone() }
                    span { class: "fs-sidebar__label", "{item.label}" }
                    if let Some(b) = &item.badge {
                        span { class: "fs-sidebar__badge", "{b}" }
                    }
                }
            }
        }
    }
}

// ── Sidebar ───────────────────────────────────────────────────────────────────

/// The ONE sidebar component for FreeSynergy.
///
/// # Content
///
/// Pass `items` for a simple flat list (treated as one untitled section) or
/// `sections` for multiple labelled groups.  If both are provided, `sections`
/// wins.  `pinned_items` are always shown at the bottom.
///
/// # Modes
///
/// - `SidebarMode::Overlay` (default) — bookmark-drawer with icon tab-strip.
///   Inject `FS_SIDEBAR_CSS` once at the app root.
/// - `SidebarMode::Panel` — static, always visible, part of the flex layout.
///   No additional CSS needed beyond what `FS_SIDEBAR_CSS` already covers.
///
/// # Examples
///
/// ```rust
/// // Shell (overlay)
/// Sidebar {
///     sections: default_sidebar_sections(),
///     pinned_items: pinned_items(),
///     active_id,
///     on_select,
///     on_context_menu: toggle_pin,
/// }
///
/// // Inner panel (static)
/// Sidebar {
///     mode: SidebarMode::Panel,
///     sections: vec![
///         SidebarSection::new("Installed", lang_items),
///     ],
///     active_id,
///     on_select,
/// }
/// ```
#[component]
pub fn Sidebar(
    // ── Content ───────────────────────────────────────────────────────────
    /// Flat item list — shorthand when no section grouping is needed.
    #[props(default)]
    items: Vec<SidebarItem>,
    /// Grouped sections with optional labels.  Overrides `items` when non-empty.
    #[props(default)]
    sections: Vec<SidebarSection>,
    /// Items pinned at the bottom of the sidebar.
    #[props(default)]
    pinned_items: Vec<SidebarItem>,

    // ── Selection ─────────────────────────────────────────────────────────
    active_id: String,
    on_select: EventHandler<String>,

    // ── Layout & behaviour ────────────────────────────────────────────────
    /// Rendering mode — Overlay (default) or Panel.
    #[props(default)]
    mode: SidebarMode,
    /// Which edge the sidebar attaches to (Overlay mode only).
    #[props(default)]
    side: SidebarSide,
    /// Panel width in pixels (default 220).
    #[props(default = 220.0_f64)]
    panel_width: f64,
    /// When `true`, adds `.fs-sidebar--open` to keep the overlay expanded
    /// regardless of hover state (use during drag-resize).
    #[props(default = false)]
    force_open: bool,

    // ── Custom content ────────────────────────────────────────────────────
    /// Optional custom panel body.  When provided, replaces the built-in item
    /// list while the sidebar frame (position, slide, tab-strip) stays intact.
    #[props(default)]
    custom_panel: Option<Element>,

    // ── Interaction ───────────────────────────────────────────────────────
    /// Called with the item ID when the user right-clicks a leaf item.
    #[props(default)]
    on_context_menu: Option<EventHandler<String>>,
) -> Element {
    // ── Resolve effective sections ────────────────────────────────────────
    let effective_sections: Vec<SidebarSection> = if !sections.is_empty() {
        sections.clone()
    } else if !items.is_empty() {
        vec![SidebarSection::untitled(items.clone())]
    } else {
        vec![]
    };

    match mode {
        SidebarMode::Panel => render_panel(
            effective_sections,
            pinned_items,
            active_id,
            on_select,
            on_context_menu,
            panel_width,
        ),
        SidebarMode::Overlay => render_overlay(
            effective_sections,
            pinned_items,
            active_id,
            on_select,
            on_context_menu,
            side,
            panel_width,
            force_open,
            custom_panel,
        ),
    }
}

// ── Panel renderer ────────────────────────────────────────────────────────────

fn render_panel(
    sections: Vec<SidebarSection>,
    pinned_items: Vec<SidebarItem>,
    active_id: String,
    on_select: EventHandler<String>,
    on_context_menu: Option<EventHandler<String>>,
    panel_width: f64,
) -> Element {
    let has_pinned = !pinned_items.is_empty();

    // Folder state for the main area (single shared state across all sections).
    let mut open_folder: Signal<Option<String>> = use_signal(|| None);
    let folder_id = open_folder.read().clone();
    let in_folder = folder_id.is_some();

    // In folder mode: find and show the open folder's children.
    let (show_sections, back_label) = if let Some(ref fid) = folder_id {
        // Find the folder across all sections.
        let mut found: Option<(Vec<SidebarItem>, String)> = None;
        'outer: for s in &sections {
            for item in &s.items {
                if item.is_folder() && &item.id == fid {
                    found = Some((item.children.clone(), item.label.clone()));
                    break 'outer;
                }
            }
        }
        if let Some((children, label)) = found {
            // Drill-in: show children as one untitled section.
            (vec![SidebarSection::untitled(children)], label)
        } else {
            (sections.clone(), String::new())
        }
    } else {
        (sections.clone(), String::new())
    };

    let level_class = if in_folder {
        "fs-sidebar__level--folder"
    } else {
        "fs-sidebar__level--root"
    };

    rsx! {
        nav {
            class: "fs-sidebar fs-sidebar--panel",
            style: "--fs-sidebar-panel-width: {panel_width}px;",

            div { class: "fs-sidebar__scroll",
                div { class: "{level_class}",
                    if in_folder {
                        button {
                            class: "fs-sidebar__item fs-sidebar__back",
                            title: "Back",
                            onclick: move |_| open_folder.set(None),
                            span { class: "fs-sidebar__icon",
                                   dangerous_inner_html: CHEVRON_LEFT }
                            span { class: "fs-sidebar__label", "{back_label}" }
                        }
                        div { class: "fs-sidebar__divider" }
                    }

                    for section in &show_sections {
                        if let Some(label) = &section.label {
                            div { class: "fs-sidebar__section-label", "{label}" }
                        }
                        SidebarItemList {
                            items:           section.items.clone(),
                            active_id:       active_id.clone(),
                            in_folder,
                            on_select:       move |id| on_select.call(id),
                            on_enter:        move |fid| open_folder.set(Some(fid)),
                            on_context_menu: on_context_menu.clone(),
                        }
                    }
                }
            }

            if has_pinned {
                div { class: "fs-sidebar__pinned",
                    SidebarItemList {
                        items:           pinned_items,
                        active_id:       active_id.clone(),
                        in_folder:       false,
                        on_select:       move |id| on_select.call(id),
                        on_enter:        move |_| {},
                        on_context_menu: None,
                    }
                }
            }
        }
    }
}

// ── Overlay renderer ──────────────────────────────────────────────────────────

fn render_overlay(
    sections: Vec<SidebarSection>,
    pinned_items: Vec<SidebarItem>,
    active_id: String,
    on_select: EventHandler<String>,
    on_context_menu: Option<EventHandler<String>>,
    side: SidebarSide,
    panel_width: f64,
    force_open: bool,
    custom_panel: Option<Element>,
) -> Element {
    // Folder state — main section.
    let mut main_open_folder: Signal<Option<String>> = use_signal(|| None);
    // Folder state — pinned section (independent).
    let mut pinned_open_folder: Signal<Option<String>> = use_signal(|| None);

    // ── Main section ──────────────────────────────────────────────────────
    // Flatten all items across sections for folder lookup and tab-strip.
    let all_items: Vec<SidebarItem> = sections
        .iter()
        .flat_map(|s| s.items.iter().cloned())
        .collect();

    let main_folder_id = main_open_folder.read().clone();
    let main_in_folder = main_folder_id.is_some();

    let (main_show_sections, main_back_label) = if let Some(ref fid) = main_folder_id {
        match all_items.iter().find(|i| &i.id == fid) {
            Some(folder) => (
                vec![SidebarSection::untitled(folder.children.clone())],
                folder.label.clone(),
            ),
            None => (sections.clone(), String::new()),
        }
    } else {
        (sections.clone(), String::new())
    };

    let main_level_class = if main_in_folder {
        "fs-sidebar__level--folder"
    } else {
        "fs-sidebar__level--root"
    };

    // ── Pinned section ────────────────────────────────────────────────────
    let pinned_folder_id = pinned_open_folder.read().clone();
    let pinned_in_folder = pinned_folder_id.is_some();

    let (pinned_show_items, pinned_back_label) = if let Some(ref fid) = pinned_folder_id {
        match pinned_items.iter().find(|i| &i.id == fid) {
            Some(folder) => (folder.children.clone(), folder.label.clone()),
            None => (pinned_items.clone(), String::new()),
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
    let has_custom = custom_panel.is_some();
    let tab_items = resolve_display_items(&all_items);

    let side_class = match side {
        SidebarSide::Left => "fs-sidebar--left",
        SidebarSide::Right => "fs-sidebar--right",
    };
    let open_class = if force_open { "fs-sidebar--open" } else { "" };

    rsx! {
        nav {
            class: "fs-sidebar {side_class} {open_class}",
            style: "--fs-sidebar-panel-width: {panel_width}px;",

            // ── Panel ─────────────────────────────────────────────────────
            div { class: "fs-sidebar__panel",

                if has_custom {
                    {custom_panel}
                } else {
                    div { class: "fs-sidebar__scroll",
                        div { class: "fs-sidebar__rail",
                            div { class: "{main_level_class}",
                                if main_in_folder {
                                    button {
                                        class: "fs-sidebar__item fs-sidebar__back",
                                        title: "Back",
                                        onclick: move |_| main_open_folder.set(None),
                                        span { class: "fs-sidebar__icon",
                                               dangerous_inner_html: CHEVRON_LEFT }
                                        span { class: "fs-sidebar__label", "{main_back_label}" }
                                    }
                                    div { class: "fs-sidebar__divider" }
                                }

                                for section in &main_show_sections {
                                    if let Some(label) = &section.label {
                                        div { class: "fs-sidebar__section-label", "{label}" }
                                    }
                                    SidebarItemList {
                                        items:           section.items.clone(),
                                        active_id:       active_id.clone(),
                                        in_folder:       main_in_folder,
                                        on_select:       move |id| on_select.call(id),
                                        on_enter:        move |fid| main_open_folder.set(Some(fid)),
                                        on_context_menu: on_context_menu.clone(),
                                    }
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
                                        span { class: "fs-sidebar__icon",
                                               dangerous_inner_html: CHEVRON_LEFT }
                                        span { class: "fs-sidebar__label", "{pinned_back_label}" }
                                    }
                                    div { class: "fs-sidebar__divider" }
                                }
                                SidebarItemList {
                                    items:           pinned_show_items,
                                    active_id:       active_id.clone(),
                                    in_folder:       pinned_in_folder,
                                    on_select:       move |id| on_select.call(id),
                                    on_enter:        move |fid| pinned_open_folder.set(Some(fid)),
                                    on_context_menu: on_context_menu.clone(),
                                }
                            }
                        }
                    }
                }
            }

            // ── Tab outer: icon pill bubbles ───────────────────────────────
            div { class: "fs-sidebar__tab-outer",

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
                                        SidebarIcon { icon: icon.clone() }
                                        span { class: "fs-sidebar__tab-label", "{label}" }
                                    }
                                }
                            }
                        }
                    }
                }

                div { class: "fs-sidebar__tab-gap" }

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
                                        SidebarIcon { icon: icon.clone() }
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

/// Standalone sidebar navigation button with icon + label and optional active
/// left-border indicator.  Use inside custom `custom_panel` content or wherever
/// a single styled button is needed without the full Sidebar frame.
#[component]
pub fn SidebarNavBtn(
    label: String,
    icon: String,
    is_active: bool,
    on_click: EventHandler,
    #[props(default = false)] left_border: bool,
) -> Element {
    let bg = if is_active {
        "var(--fs-color-bg-overlay, #1e293b)"
    } else {
        "transparent"
    };
    let color = if is_active {
        "var(--fs-color-primary, #06b6d4)"
    } else {
        "var(--fs-color-text-primary, #e2e8f0)"
    };
    let weight = if is_active { "600" } else { "400" };
    let border_left = if left_border {
        if is_active {
            "2px solid var(--fs-color-primary, #06b6d4)"
        } else {
            "2px solid transparent"
        }
    } else {
        "none"
    };
    rsx! {
        button {
            style: "display: flex; align-items: center; gap: 10px; width: 100%; \
                    padding: 8px 12px; border: none; border-left: {border_left}; \
                    border-radius: var(--fs-radius-md, 6px); cursor: pointer; \
                    font-size: 14px; text-align: left; background: {bg}; \
                    color: {color}; font-weight: {weight}; margin-bottom: 2px; \
                    font-family: inherit;",
            onclick: move |_| on_click.call(()),
            span { style: "font-size: 16px;", "{icon}" }
            span { "{label}" }
        }
    }
}

// ── FsTabView ─────────────────────────────────────────────────────────────────

/// A tab definition for `FsTabView`.
#[derive(Clone, PartialEq, Debug)]
pub struct FsTabDef {
    pub id: String,
    pub label: String,
    /// Optional SVG markup or emoji icon shown left of the label.
    pub icon: Option<String>,
}

impl FsTabDef {
    pub fn new(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            icon: None,
        }
    }
    pub fn with_icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = Some(icon.into());
        self
    }
}

/// CSS for `FsTabView` — inject once at the app root via `style { FS_TAB_VIEW_CSS }`.
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
    font-family: inherit;
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
/// slides in from the right with a blur-to-sharp effect.  Navigating left
/// reverses the direction.  Inject `FS_TAB_VIEW_CSS` once at the app root.
#[component]
pub fn FsTabView(
    tabs: Vec<FsTabDef>,
    active_id: String,
    on_change: EventHandler<String>,
    children: Element,
) -> Element {
    let mut prev_id: Signal<String> = use_signal(|| active_id.clone());
    let mut content_key: Signal<u32> = use_signal(|| 0);
    let mut anim_class: Signal<&'static str> = use_signal(|| "");

    {
        let new_id = active_id.clone();
        let t_clone = tabs.clone();
        use_effect(move || {
            let prev = prev_id.peek().clone();
            if prev != new_id {
                let pi = t_clone.iter().position(|t| t.id == prev).unwrap_or(0);
                let ai = t_clone.iter().position(|t| t.id == new_id).unwrap_or(0);
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

    let ck = *content_key.read();
    let cls = *anim_class.read();

    rsx! {
        div { class: "fs-tab-view",
            div { class: "fs-tab-view__bar",
                for tab in &tabs {
                    {
                        let is_active = tab.id == active_id;
                        let tid = tab.id.clone();
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
            div {
                key:   "{ck}",
                class: "fs-tab-view__content {cls}",
                {children}
            }
        }
    }
}
