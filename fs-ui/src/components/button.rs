use dioxus::prelude::*;

/// Button visual variant.
#[derive(Clone, PartialEq, Default)]
pub enum ButtonVariant {
    #[default]
    Primary,
    Secondary,
    Danger,
}

impl ButtonVariant {
    fn css_class(&self) -> &'static str {
        match self {
            ButtonVariant::Primary => "fs-button--primary",
            ButtonVariant::Secondary => "fs-button--secondary",
            ButtonVariant::Danger => "fs-button--danger",
        }
    }
}

/// Props for [`Button`].
#[derive(Props, Clone, PartialEq)]
pub struct ButtonProps {
    /// Button label text.
    pub label: String,
    /// Visual variant (primary / secondary / danger).
    #[props(default)]
    pub variant: ButtonVariant,
    /// Disabled state.
    #[props(default = false)]
    pub disabled: bool,
    /// Optional click handler.
    pub onclick: Option<EventHandler<MouseEvent>>,
    /// Optional extra CSS classes.
    #[props(default)]
    pub class: String,
}

/// A styled button following FreeSynergy design.
#[component]
pub fn Button(props: ButtonProps) -> Element {
    let variant_class = props.variant.css_class();
    rsx! {
        button {
            class: "fs-button {variant_class} {props.class}",
            disabled: props.disabled,
            onclick: move |e| {
                if let Some(h) = &props.onclick {
                    h.call(e);
                }
            },
            "{props.label}"
        }
    }
}

#[cfg(feature = "tui")]
mod tui {
    // TUI fallback stub
}
