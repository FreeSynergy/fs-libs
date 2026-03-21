// fsn-components/modal.rs — Modal overlay dialog and floating Window panel.

use dioxus::prelude::*;

// ── Modal ─────────────────────────────────────────────────────────────────────

#[derive(Props, Clone, PartialEq)]
pub struct ModalProps {
    /// Whether the modal is visible.
    pub open: bool,
    /// Title shown in the modal header.
    pub title: String,
    /// Body content.
    pub children: Element,
    /// Called when the user closes the modal (X button or backdrop click).
    #[props(default)]
    pub on_close: EventHandler,
    /// Optional CSS width for the dialog panel (e.g. `"480px"`). Defaults to `"480px"`.
    pub width: Option<String>,
}

/// Centered overlay dialog with a semi-transparent backdrop.
///
/// The modal renders on top of all content via `position: fixed; z-index: 1000`.
/// The body is scrollable. Backdrop click triggers `on_close`.
///
/// ```no_run
/// Modal {
///     open: show_modal,
///     title: "Confirm deploy",
///     on_close: move |_| show_modal.set(false),
///     p { "Are you sure?" }
/// }
/// ```
#[component]
pub fn Modal(props: ModalProps) -> Element {
    if !props.open {
        return rsx! { Fragment {} };
    }

    let width = props.width.as_deref().unwrap_or("480px");

    rsx! {
        // Backdrop
        div {
            class: "fsn-modal-backdrop",
            style: "position: fixed; inset: 0; z-index: 1000; \
                    background: rgba(0,0,0,0.65); \
                    display: flex; align-items: center; justify-content: center;",
            onclick: move |_| props.on_close.call(()),

            // Panel (stop click propagation so backdrop click doesn't bubble from panel)
            div {
                class: "fsn-modal-panel",
                style: "width: {width}; max-width: calc(100vw - 32px); max-height: 85vh; \
                        display: flex; flex-direction: column; border-radius: 12px; \
                        background: rgba(22,27,34,0.85); \
                        backdrop-filter: blur(var(--glass-blur,12px)); \
                        -webkit-backdrop-filter: blur(var(--glass-blur,12px)); \
                        border: 1px solid var(--fsn-color-border-default, #334155); \
                        box-shadow: 0 16px 48px rgba(0,0,0,0.7); \
                        overflow: hidden;",
                onclick: move |e| e.stop_propagation(),

                // Header
                div {
                    class: "fsn-modal-header",
                    style: "display: flex; align-items: center; justify-content: space-between; \
                            padding: 14px 18px; \
                            border-bottom: 1px solid var(--fsn-color-border-default, #334155); \
                            flex-shrink: 0;",

                    span {
                        style: "font-size: 14px; font-weight: 600; \
                                color: var(--fsn-color-text-primary, #e2e8f0);",
                        "{props.title}"
                    }

                    button {
                        aria_label: "Close dialog",
                        style: "background: none; border: none; cursor: pointer; padding: 2px 6px; \
                                font-size: 18px; line-height: 1; \
                                color: var(--fsn-color-text-muted, #64748b); \
                                border-radius: 4px;",
                        onclick: move |_| props.on_close.call(()),
                        "×"
                    }
                }

                // Scrollable body
                div {
                    class: "fsn-modal-body",
                    style: "padding: 18px; overflow-y: auto; flex: 1;",
                    {props.children}
                }
            }
        }
    }
}

// ── Window ────────────────────────────────────────────────────────────────────

#[derive(Props, Clone, PartialEq)]
pub struct WindowProps {
    /// Whether the window is visible.
    pub open: bool,
    /// Title shown in the window header.
    pub title: String,
    /// Body content.
    pub children: Element,
    /// Called when the user closes the window.
    #[props(default)]
    pub on_close: EventHandler,
    /// Optional CSS width. Defaults to `"400px"`.
    pub width: Option<String>,
}

/// Floating panel without a backdrop. Useful for non-blocking side panels or
/// inspector views that do not interrupt the main workflow.
///
/// ```no_run
/// Window {
///     open: show_panel,
///     title: "Details",
///     on_close: move |_| show_panel.set(false),
///     p { "Selected item info…" }
/// }
/// ```
#[component]
pub fn Window(props: WindowProps) -> Element {
    if !props.open {
        return rsx! { Fragment {} };
    }

    let width = props.width.as_deref().unwrap_or("400px");

    rsx! {
        div {
            class: "fsn-window",
            style: "width: {width}; max-width: calc(100vw - 32px); max-height: 80vh; \
                    display: flex; flex-direction: column; border-radius: 12px; \
                    background: rgba(22,27,34,0.85); \
                    backdrop-filter: blur(var(--glass-blur,12px)); \
                    -webkit-backdrop-filter: blur(var(--glass-blur,12px)); \
                    border: 1px solid var(--fsn-color-border-default, #334155); \
                    box-shadow: 0 8px 32px rgba(0,0,0,0.5); \
                    overflow: hidden;",

            // Header
            div {
                class: "fsn-window-header",
                style: "display: flex; align-items: center; justify-content: space-between; \
                        padding: 12px 16px; \
                        border-bottom: 1px solid var(--fsn-color-border-default, #334155); \
                        flex-shrink: 0;",

                span {
                    style: "font-size: 13px; font-weight: 600; \
                            color: var(--fsn-color-text-primary, #e2e8f0);",
                    "{props.title}"
                }

                button {
                    aria_label: "Close panel",
                    style: "background: none; border: none; cursor: pointer; padding: 2px 6px; \
                            font-size: 16px; line-height: 1; \
                            color: var(--fsn-color-text-muted, #64748b); \
                            border-radius: 4px;",
                    onclick: move |_| props.on_close.call(()),
                    "×"
                }
            }

            // Scrollable body
            div {
                class: "fsn-window-body",
                style: "padding: 16px; overflow-y: auto; flex: 1;",
                {props.children}
            }
        }
    }
}
