use dioxus::prelude::*;

/// Position of the tooltip bubble relative to the trigger element.
#[derive(Clone, PartialEq, Default)]
pub enum TooltipPosition {
    #[default]
    Top,
    Bottom,
    Left,
    Right,
}

impl TooltipPosition {
    fn css_class(&self) -> &'static str {
        match self {
            TooltipPosition::Top => "fs-tooltip--top",
            TooltipPosition::Bottom => "fs-tooltip--bottom",
            TooltipPosition::Left => "fs-tooltip--left",
            TooltipPosition::Right => "fs-tooltip--right",
        }
    }
}

/// Props for [`Tooltip`].
#[derive(Props, Clone, PartialEq)]
pub struct TooltipProps {
    /// Tooltip bubble text.
    pub text: String,
    /// Position relative to the trigger.
    #[props(default)]
    pub position: TooltipPosition,
    /// Optional extra CSS classes.
    #[props(default)]
    pub class: String,
    /// The element that triggers the tooltip on hover.
    children: Element,
}

/// Wraps content with a CSS-powered tooltip bubble.
#[component]
pub fn Tooltip(props: TooltipProps) -> Element {
    let pos_class = props.position.css_class();
    rsx! {
        span {
            class: "fs-tooltip-anchor {pos_class} {props.class}",
            {props.children}
            span {
                class: "fs-tooltip__bubble",
                role: "tooltip",
                "{props.text}"
            }
        }
    }
}

#[cfg(feature = "tui")]
mod tui {
    // TUI fallback stub
}
