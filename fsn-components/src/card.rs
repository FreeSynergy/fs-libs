// fsn-components/card.rs — Card, Badge, Divider, Spinner, Tooltip.

use dioxus::prelude::*;

// ── Card ──────────────────────────────────────────────────────────────────────

#[derive(Props, Clone, PartialEq)]
pub struct CardProps {
    pub children: Element,
    /// Optional extra inline CSS.
    pub extra_style: Option<String>,
    /// Applies glassmorphism backdrop-filter (requires glass.css variables).
    #[props(default = false)]
    pub glass: bool,
}

/// Elevated surface container with optional glassmorphism.
#[component]
pub fn Card(props: CardProps) -> Element {
    let glass_css = if props.glass {
        "background: rgba(22,27,34,var(--glass-bg-opacity,0.08)); \
         backdrop-filter: blur(var(--glass-blur,12px)); \
         -webkit-backdrop-filter: blur(var(--glass-blur,12px)); \
         border: 1px solid rgba(48,54,61,var(--glass-border-opacity,0.15));"
    } else {
        "background: var(--fsn-color-bg-surface, #1e293b); \
         border: 1px solid var(--fsn-color-border-default, #334155);"
    };

    let style = format!(
        "border-radius: 12px; padding: 16px; \
         box-shadow: var(--shadow-md, 0 4px 12px rgba(0,0,0,0.5)); \
         {}{}",
        glass_css,
        props.extra_style.as_deref().unwrap_or("")
    );

    rsx! {
        div {
            class: "fsn-card",
            style: "{style}",
            {props.children}
        }
    }
}

// ── Badge ─────────────────────────────────────────────────────────────────────

/// Color variant for a badge.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BadgeVariant {
    #[default]
    Default,
    Success,
    Warning,
    Error,
    Info,
}

impl BadgeVariant {
    fn style(self) -> &'static str {
        match self {
            Self::Default => "background: rgba(100,116,139,0.2); color: #94a3b8;",
            Self::Success => "background: rgba(63,185,80,0.15);  color: #3fb950;",
            Self::Warning => "background: rgba(210,153,34,0.15);  color: #d29922;",
            Self::Error   => "background: rgba(248,81,73,0.15);   color: #f85149;",
            Self::Info    => "background: rgba(0,188,212,0.15);   color: #00bcd4;",
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct BadgeProps {
    pub children: Element,
    #[props(default)]
    pub variant: BadgeVariant,
}

/// Small inline status indicator.
#[component]
pub fn Badge(props: BadgeProps) -> Element {
    let style = format!(
        "display: inline-block; padding: 2px 8px; border-radius: 999px; \
         font-size: 11px; font-weight: 600; line-height: 1.6; {}",
        props.variant.style()
    );
    rsx! {
        span {
            class: "fsn-badge",
            style: "{style}",
            {props.children}
        }
    }
}

// ── Divider ───────────────────────────────────────────────────────────────────

#[derive(Props, Clone, PartialEq)]
pub struct DividerProps {
    /// Optional label shown in the middle of the divider.
    pub label: Option<String>,
    #[props(default = "16px 0".to_string())]
    pub margin: String,
}

/// Horizontal rule with an optional centred label.
#[component]
pub fn Divider(props: DividerProps) -> Element {
    rsx! {
        div {
            role: "separator",
            class: "fsn-divider",
            style: "display: flex; align-items: center; gap: 10px; margin: {props.margin};",

            div {
                style: "flex: 1; height: 1px; \
                        background: var(--fsn-color-border-default, #334155);",
            }

            if let Some(label) = &props.label {
                span {
                    style: "font-size: 11px; color: var(--fsn-color-text-muted, #64748b); \
                            white-space: nowrap; flex-shrink: 0;",
                    "{label}"
                }
                div {
                    style: "flex: 1; height: 1px; \
                            background: var(--fsn-color-border-default, #334155);",
                }
            }
        }
    }
}

// ── Spinner ───────────────────────────────────────────────────────────────────

#[derive(Props, Clone, PartialEq)]
pub struct SpinnerProps {
    /// Size in pixels.
    #[props(default = 24_u32)]
    pub size: u32,
    /// Accessible loading label (announced by screen readers).
    #[props(default = "Loading…".to_string())]
    pub label: String,
}

/// Animated circular loading indicator (raw pixel size).
#[component]
pub fn Spinner(props: SpinnerProps) -> Element {
    let border_w = (props.size / 8).max(2);
    let style = format!(
        "display: inline-block; width: {}px; height: {}px; border-radius: 50%; \
         border: {}px solid var(--fsn-color-border-default, #334155); \
         border-top-color: var(--fsn-color-primary, #06b6d4); \
         animation: fsn-spin 0.7s linear infinite;",
        props.size, props.size, border_w
    );
    rsx! {
        span {
            role: "status",
            aria_label: "{props.label}",
            class: "fsn-spinner",
            style: "{style}",
        }
        style {
            "@keyframes fsn-spin {{ from {{ transform: rotate(0deg); }} to {{ transform: rotate(360deg); }} }}"
        }
    }
}

// ── SpinnerSize ───────────────────────────────────────────────────────────────

/// Named size for [`LoadingSpinner`].
#[derive(Clone, PartialEq, Debug, Default)]
pub enum SpinnerSize {
    /// 16 px — inline / button context.
    Sm,
    /// 24 px — default for content areas.
    #[default]
    Md,
    /// 40 px — full-page / overlay loading.
    Lg,
}

impl SpinnerSize {
    fn px(&self) -> u32 {
        match self {
            SpinnerSize::Sm => 16,
            SpinnerSize::Md => 24,
            SpinnerSize::Lg => 40,
        }
    }
}

// ── LoadingSpinner ────────────────────────────────────────────────────────────

/// Animated spinner with named sizes (Sm / Md / Lg).
///
/// # Example
/// ```rust
/// rsx! { LoadingSpinner {} }
/// rsx! { LoadingSpinner { size: SpinnerSize::Lg } }
/// ```
#[component]
pub fn LoadingSpinner(
    #[props(default)]
    size: SpinnerSize,
    #[props(default = "Loading…".to_string())]
    label: String,
) -> Element {
    rsx! { Spinner { size: size.px(), label } }
}

// ── LoadingOverlay ────────────────────────────────────────────────────────────

/// Centred loading overlay with spinner and optional text message.
#[component]
pub fn LoadingOverlay(
    #[props(default = SpinnerSize::Lg)]
    size: SpinnerSize,
    message: Option<String>,
) -> Element {
    rsx! {
        div {
            style: "display: flex; flex-direction: column; align-items: center; \
                    justify-content: center; gap: 12px; padding: 48px; \
                    color: var(--fsn-color-text-muted, #94a3b8);",
            LoadingSpinner { size }
            if let Some(msg) = message {
                span { style: "font-size: 13px;", "{msg}" }
            }
        }
    }
}

// ── Tooltip ───────────────────────────────────────────────────────────────────

#[derive(Props, Clone, PartialEq)]
pub struct TooltipProps {
    /// The trigger element.
    pub children: Element,
    /// Tooltip content text.
    pub text: String,
}

/// Wraps any element with a CSS tooltip on hover (no JS required).
///
/// The tooltip is implemented via `::after` using `data-tip`, so no absolute
/// positioning logic is needed. For complex popover content, use a dedicated
/// overlay component.
#[component]
pub fn Tooltip(props: TooltipProps) -> Element {
    rsx! {
        span {
            class: "fsn-tooltip",
            style: "position: relative; display: inline-flex;",
            title: "{props.text}",  // native fallback for accessibility

            {props.children}

            span {
                class: "fsn-tooltip__bubble",
                aria_hidden: "true",
                style: "pointer-events: none; opacity: 0; position: absolute; \
                        bottom: calc(100% + 6px); left: 50%; transform: translateX(-50%); \
                        white-space: nowrap; padding: 4px 10px; border-radius: 4px; \
                        font-size: 11px; font-weight: 500; z-index: 1000; \
                        background: var(--fsn-color-bg-sidebar, #0f172a); \
                        color: var(--fsn-color-text-primary, #e2e8f0); \
                        border: 1px solid var(--fsn-color-border-default, #334155); \
                        box-shadow: 0 2px 8px rgba(0,0,0,0.4); \
                        transition: opacity 0.15s;",
                "{props.text}"
            }
        }

        // Tooltip hover styles via a <style> block.
        style {
            ".fsn-tooltip:hover .fsn-tooltip__bubble {{ opacity: 1; }}"
        }
    }
}
