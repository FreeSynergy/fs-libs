// fsn-components/button.rs — Button component with variants, sizes, and loading state.

use dioxus::prelude::*;

// ── ButtonVariant ─────────────────────────────────────────────────────────────

/// Visual style variant for a button.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ButtonVariant {
    /// Filled with the primary accent color.
    #[default]
    Primary,
    /// Outlined with subdued styling.
    Secondary,
    /// Text-only, no background or border.
    Ghost,
    /// Destructive action — red accent.
    Danger,
}

impl ButtonVariant {
    fn css(self) -> &'static str {
        match self {
            Self::Primary   => "background: var(--fsn-color-primary, #06b6d4); \
                                color: var(--fsn-color-primary-text, #000); \
                                border: none;",
            Self::Secondary => "background: transparent; \
                                color: var(--fsn-color-text-primary, #e2e8f0); \
                                border: 1px solid var(--fsn-color-border-default, #334155);",
            Self::Ghost     => "background: transparent; \
                                color: var(--fsn-color-text-secondary, #94a3b8); \
                                border: none;",
            Self::Danger    => "background: var(--fsn-color-error, #ef4444); \
                                color: #fff; \
                                border: none;",
        }
    }

    /// CSS class suffix for aria / test selection.
    pub fn class_suffix(self) -> &'static str {
        match self {
            Self::Primary   => "primary",
            Self::Secondary => "secondary",
            Self::Ghost     => "ghost",
            Self::Danger    => "danger",
        }
    }
}

// ── ButtonSize ────────────────────────────────────────────────────────────────

/// Size variant for a button.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ButtonSize {
    Sm,
    #[default]
    Md,
    Lg,
}

impl ButtonSize {
    fn css(self) -> &'static str {
        match self {
            Self::Sm => "padding: 4px 10px; font-size: 12px; border-radius: 4px;",
            Self::Md => "padding: 7px 16px; font-size: 13px; border-radius: 6px;",
            Self::Lg => "padding: 10px 24px; font-size: 15px; border-radius: 8px;",
        }
    }
}

// ── Button ────────────────────────────────────────────────────────────────────

#[derive(Props, Clone, PartialEq)]
pub struct ButtonProps {
    /// Button label text.
    pub children: Element,
    #[props(default)]
    pub variant: ButtonVariant,
    #[props(default)]
    pub size: ButtonSize,
    /// Shows a spinner and disables the button when true.
    #[props(default = false)]
    pub loading: bool,
    #[props(default = false)]
    pub disabled: bool,
    /// Icon rendered to the left of the label.
    pub left_icon: Option<Element>,
    /// Icon rendered to the right of the label.
    pub right_icon: Option<Element>,
    /// Click handler.
    #[props(default)]
    pub onclick: EventHandler<MouseEvent>,
    /// Optional aria-label override (defaults to button text).
    pub aria_label: Option<String>,
    /// Additional inline CSS appended after base styles.
    pub extra_style: Option<String>,
}

/// Primary interaction element with variant, size, loading, and icon support.
///
/// ```no_run
/// Button { variant: ButtonVariant::Primary, onclick: move |_| { /* … */ },
///     "Deploy"
/// }
/// ```
#[component]
pub fn Button(props: ButtonProps) -> Element {
    let is_disabled = props.disabled || props.loading;
    let base = format!(
        "display: inline-flex; align-items: center; justify-content: center; gap: 6px; \
         cursor: {}; font-family: inherit; font-weight: 500; \
         transition: opacity 0.15s, background 0.15s; \
         {} {}{}",
        if is_disabled { "not-allowed" } else { "pointer" },
        props.variant.css(),
        props.size.css(),
        props.extra_style.as_deref().map(|s| format!(" {s}")).unwrap_or_default(),
    );

    let variant_class = format!("fsn-btn fsn-btn--{}", props.variant.class_suffix());

    rsx! {
        button {
            class: "{variant_class}",
            style: "{base}",
            disabled: is_disabled,
            aria_label: props.aria_label.as_deref(),
            aria_busy: if props.loading { "true" } else { "false" },
            onclick: move |e| {
                if !is_disabled {
                    props.onclick.call(e);
                }
            },

            // Left icon
            if let Some(icon) = &props.left_icon {
                span { aria_hidden: "true", {icon.clone()} }
            }

            // Loading spinner replaces the left icon when active
            if props.loading {
                span {
                    style: "width: 12px; height: 12px; border-radius: 50%; \
                            border: 2px solid currentColor; border-top-color: transparent; \
                            animation: fsn-spin 0.6s linear infinite; display: inline-block;",
                    aria_hidden: "true",
                }
            }

            {props.children}

            // Right icon
            if let Some(icon) = &props.right_icon {
                span { aria_hidden: "true", {icon.clone()} }
            }
        }

        // Spinner keyframes (injected once, idempotent in practice)
        style {
            "@keyframes fsn-spin {{ from {{ transform: rotate(0deg); }} to {{ transform: rotate(360deg); }} }}"
        }
    }
}

// ── IconButton ────────────────────────────────────────────────────────────────

#[derive(Props, Clone, PartialEq)]
pub struct IconButtonProps {
    /// Icon element rendered inside the button.
    pub icon: Element,
    #[props(default)]
    pub variant: ButtonVariant,
    #[props(default)]
    pub size: ButtonSize,
    /// Shows a spinner and disables the button when true.
    #[props(default = false)]
    pub loading: bool,
    #[props(default = false)]
    pub disabled: bool,
    /// Click handler.
    #[props(default)]
    pub onclick: EventHandler<MouseEvent>,
    /// Accessible label (required for icon-only buttons).
    pub aria_label: String,
}

/// Square icon-only button with the same variant and size options as `Button`.
///
/// No text label is rendered — the `aria_label` is required for accessibility.
///
/// ```no_run
/// IconButton {
///     icon: rsx! { span { "🗑" } },
///     variant: ButtonVariant::Danger,
///     aria_label: "Delete item",
///     onclick: move |_| { /* … */ },
/// }
/// ```
#[component]
pub fn IconButton(props: IconButtonProps) -> Element {
    let is_disabled = props.disabled || props.loading;

    // Square padding: override the horizontal padding from ButtonSize
    let pad = match props.size {
        ButtonSize::Sm => "padding: 4px;",
        ButtonSize::Md => "padding: 7px;",
        ButtonSize::Lg => "padding: 10px;",
    };
    let font = match props.size {
        ButtonSize::Sm => "font-size: 12px; border-radius: 4px;",
        ButtonSize::Md => "font-size: 14px; border-radius: 6px;",
        ButtonSize::Lg => "font-size: 16px; border-radius: 8px;",
    };

    let base = format!(
        "display: inline-flex; align-items: center; justify-content: center; \
         cursor: {}; font-family: inherit; line-height: 1; \
         transition: opacity 0.15s, background 0.15s; \
         {} {} {}",
        if is_disabled { "not-allowed" } else { "pointer" },
        props.variant.css(),
        pad,
        font,
    );

    let variant_class = format!("fsn-icon-btn fsn-icon-btn--{}", props.variant.class_suffix());

    rsx! {
        button {
            class:      "{variant_class}",
            style:      "{base}",
            disabled:   is_disabled,
            aria_label: "{props.aria_label}",
            aria_busy:  if props.loading { "true" } else { "false" },
            onclick: move |e| {
                if !is_disabled {
                    props.onclick.call(e);
                }
            },

            if props.loading {
                span {
                    style: "width: 1em; height: 1em; border-radius: 50%; \
                            border: 2px solid currentColor; border-top-color: transparent; \
                            animation: fsn-spin 0.6s linear infinite; display: inline-block;",
                    aria_hidden: "true",
                }
            } else {
                span { aria_hidden: "true", {props.icon} }
            }
        }
    }
}
