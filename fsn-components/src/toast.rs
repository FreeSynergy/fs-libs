// fsn-components/toast.rs — Toast notification system: ToastProvider + use_toast hook.

use dioxus::prelude::*;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::toast_bus::{ToastLevel, ToastMessage};

// ── ToastId ───────────────────────────────────────────────────────────────────

static NEXT_ID: AtomicU64 = AtomicU64::new(1);

fn next_id() -> u64 {
    NEXT_ID.fetch_add(1, Ordering::Relaxed)
}

// ── ToastEntry ────────────────────────────────────────────────────────────────

/// A live toast entry managed by the provider.
#[derive(Clone, PartialEq)]
pub struct ToastEntry {
    pub id:      u64,
    pub level:   ToastLevel,
    pub title:   String,
    pub body:    Option<String>,
    pub timeout: Option<u32>,
}

impl ToastEntry {
    fn from_message(msg: ToastMessage) -> Self {
        Self {
            id:      next_id(),
            level:   msg.level,
            title:   msg.title,
            body:    msg.body,
            timeout: msg.timeout,
        }
    }

    fn level_style(&self) -> &'static str {
        match self.level {
            ToastLevel::Info    => "border-left: 3px solid #06b6d4;",
            ToastLevel::Success => "border-left: 3px solid #22c55e;",
            ToastLevel::Warning => "border-left: 3px solid #f59e0b;",
            ToastLevel::Error   => "border-left: 3px solid #ef4444;",
        }
    }

    fn level_icon(&self) -> &'static str {
        match self.level {
            ToastLevel::Info    => "ℹ",
            ToastLevel::Success => "✓",
            ToastLevel::Warning => "⚠",
            ToastLevel::Error   => "✗",
        }
    }
}

// ── ToastContext ──────────────────────────────────────────────────────────────

/// Shared context exposed by `ToastProvider` to child components.
///
/// `Signal<T>` is `Copy` with shared interior state, so `ToastContext` is also `Copy`.
/// Methods take `&mut self` as required by `Signal::write()`.
#[derive(Clone, Copy)]
pub struct ToastContext {
    entries: Signal<Vec<ToastEntry>>,
}

impl PartialEq for ToastContext {
    fn eq(&self, other: &Self) -> bool {
        self.entries == other.entries
    }
}

impl ToastContext {
    /// Push a toast from a [`ToastMessage`].
    pub fn push(&mut self, msg: ToastMessage) {
        self.entries.write().push(ToastEntry::from_message(msg));
    }

    /// Dismiss a toast by id.
    pub fn dismiss(&mut self, id: u64) {
        self.entries.write().retain(|e| e.id != id);
    }
}

// ── use_toast ─────────────────────────────────────────────────────────────────

/// Hook that returns the `ToastContext` injected by the nearest `ToastProvider`.
///
/// Panics if called outside a `ToastProvider`.
pub fn use_toast() -> ToastContext {
    use_context::<ToastContext>()
}

// ── ToastProvider ─────────────────────────────────────────────────────────────

#[derive(Props, Clone, PartialEq)]
pub struct ToastProviderProps {
    pub children: Element,
}

/// Root context provider. Wrap your app (or a subtree) with this component.
///
/// All descendants can call `use_toast()` to push notifications.
///
/// ```no_run
/// ToastProvider {
///     // … your app …
/// }
/// ```
#[component]
pub fn ToastProvider(props: ToastProviderProps) -> Element {
    let entries: Signal<Vec<ToastEntry>> = use_signal(Vec::new);
    let ctx = ToastContext { entries };
    use_context_provider(|| ctx);

    rsx! {
        {props.children}
        ToastStack { ctx }
    }
}

// ── ToastStack (internal) ─────────────────────────────────────────────────────

#[derive(Props, Clone, PartialEq)]
struct ToastStackProps {
    ctx: ToastContext,
}

#[component]
fn ToastStack(props: ToastStackProps) -> Element {
    let entries = props.ctx.entries.read().clone();

    if entries.is_empty() {
        return rsx! { Fragment {} };
    }

    rsx! {
        div {
            class: "fsn-toast-stack",
            role:  "region",
            aria_label: "Notifications",
            style: "position: fixed; bottom: 20px; right: 20px; z-index: 9999; \
                    display: flex; flex-direction: column-reverse; gap: 8px; \
                    max-width: 360px; width: 100%;",

            for entry in &entries {
                ToastItem {
                    key: "{entry.id}",
                    entry: entry.clone(),
                    ctx: props.ctx.clone(),
                }
            }
        }
    }
}

// ── ToastItem (internal) ──────────────────────────────────────────────────────

#[derive(Props, Clone, PartialEq)]
struct ToastItemProps {
    entry: ToastEntry,
    ctx:   ToastContext,
}

#[component]
fn ToastItem(props: ToastItemProps) -> Element {
    let id  = props.entry.id;
    let ctx = props.ctx;
    let style = format!(
        "display: flex; align-items: flex-start; gap: 10px; \
         padding: 12px 14px; border-radius: 8px; \
         background: var(--fsn-color-bg-sidebar, #0f172a); \
         box-shadow: 0 4px 16px rgba(0,0,0,0.6); {}",
        props.entry.level_style()
    );

    rsx! {
        div {
            class: "fsn-toast fsd-fade-in-up",
            role:  "status",
            aria_live: "polite",
            style: "{style}",

            // Icon
            span {
                style: "font-size: 14px; flex-shrink: 0; padding-top: 1px;",
                aria_hidden: "true",
                "{props.entry.level_icon()}"
            }

            // Body
            div { style: "flex: 1; min-width: 0;",
                p {
                    style: "margin: 0; font-size: 13px; font-weight: 600; \
                            color: var(--fsn-color-text-primary, #e2e8f0); \
                            overflow: hidden; text-overflow: ellipsis; white-space: nowrap;",
                    "{props.entry.title}"
                }
                if let Some(body) = &props.entry.body {
                    p {
                        style: "margin: 4px 0 0; font-size: 12px; \
                                color: var(--fsn-color-text-secondary, #94a3b8);",
                        "{body}"
                    }
                }
            }

            // Dismiss button
            button {
                aria_label: "Dismiss notification",
                style: "background: none; border: none; cursor: pointer; padding: 0; \
                        color: var(--fsn-color-text-muted, #64748b); font-size: 16px; \
                        line-height: 1; flex-shrink: 0;",
                onclick: move |_| { let mut c = ctx; c.dismiss(id); },
                "×"
            }
        }
    }
}
