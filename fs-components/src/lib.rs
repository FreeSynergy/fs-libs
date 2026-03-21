// fs-components — FreeSynergy UI component library.
//
// Provides two layers:
//
//   1. Renderer-agnostic buses (`toast_bus`) — always compiled, usable from CLI,
//      background tasks, and any renderer.
//
//   2. Dioxus UI components (all other modules) — compiled only when the
//      `dioxus` feature is enabled.
//
// Feature flags:
//   dioxus   — Dioxus components only (macro, html, signals, hooks)
//   desktop  — dioxus + dioxus/desktop
//   web      — dioxus + dioxus/web

// Always available.
pub mod toast_bus;
pub use toast_bus::{ErrorBus, ErrorMessage, ToastBus, ToastLevel, ToastMessage};

// Dioxus-gated components.
#[cfg(feature = "dioxus")]
pub mod nav;
#[cfg(feature = "dioxus")]
pub mod button;
#[cfg(feature = "dioxus")]
pub mod card;
#[cfg(feature = "dioxus")]
pub mod form_field;
#[cfg(feature = "dioxus")]
pub mod input;
#[cfg(feature = "dioxus")]
pub mod toast;
#[cfg(feature = "dioxus")]
pub mod modal;
#[cfg(feature = "dioxus")]
pub mod form;
#[cfg(feature = "dioxus")]
pub mod controls;
#[cfg(feature = "dioxus")]
pub mod display;
#[cfg(feature = "dioxus")]
pub mod layout;
#[cfg(feature = "dioxus")]
pub mod overlay;
#[cfg(feature = "dioxus")]
pub mod app;
#[cfg(feature = "dioxus")]
pub mod chat;

// ── Re-exports ─────────────────────────────────────────────────────────────────

#[cfg(feature = "dioxus")]
pub use nav::{FsSidebar, FsSidebarItem, FS_SIDEBAR_CSS, SidebarNavBtn, TabBtn,
               FsTabDef, FsTabView, FS_TAB_VIEW_CSS};
#[cfg(feature = "dioxus")]
pub use button::{Button, ButtonSize, ButtonVariant, IconButton};
#[cfg(feature = "dioxus")]
pub use card::{Badge, BadgeVariant, Card, Divider, LoadingOverlay, LoadingSpinner, Spinner, SpinnerSize, Tooltip};
#[cfg(feature = "dioxus")]
pub use form_field::FormField;
#[cfg(feature = "dioxus")]
pub use input::{Checkbox, Input, Select, SelectOption, Textarea};
#[cfg(feature = "dioxus")]
pub use toast::{use_toast, ToastContext, ToastEntry, ToastProvider};
#[cfg(feature = "dioxus")]
pub use modal::{Modal, Window};
#[cfg(feature = "dioxus")]
pub use form::{Form, FormGrid};
#[cfg(feature = "dioxus")]
pub use controls::{MultiSelect, RadioGroup, Slider, Toggle};
#[cfg(feature = "dioxus")]
pub use display::{CodeBlock, Progress, Table, TableColumn};
#[cfg(feature = "dioxus")]
pub use layout::{
    Breadcrumb, BreadcrumbItem, ScrollContainer, SearchBar, Sidebar, SidebarItem,
    SidebarSection, StatusBar, TabItem, Tabs,
};
#[cfg(feature = "dioxus")]
pub use overlay::{ContextMenu, ContextMenuEntry, HelpPanel, NotificationItem, NotificationList};
#[cfg(feature = "dioxus")]
pub use app::{AppEntry, AppLauncher, LangOption, LangSwitcher, ThemeOption, ThemeSwitcher};
#[cfg(feature = "dioxus")]
pub use chat::{ChatMessage, ChatRole, LlmChat};
