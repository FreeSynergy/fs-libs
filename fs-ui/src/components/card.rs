use dioxus::prelude::*;

/// Props for [`Card`].
#[derive(Props, Clone, PartialEq)]
pub struct CardProps {
    /// Card title.
    pub title: String,
    /// Optional subtitle rendered below the title.
    pub subtitle: Option<String>,
    /// Optional extra CSS classes.
    #[props(default)]
    pub class: String,
    /// Card body content.
    children: Element,
}

/// A content card with a title, optional subtitle, and body slot.
#[component]
pub fn Card(props: CardProps) -> Element {
    rsx! {
        div { class: "fs-card {props.class}",
            div { class: "fs-card__header",
                h3 { class: "fs-card__title", "{props.title}" }
                if let Some(sub) = &props.subtitle {
                    p { class: "fs-card__subtitle", "{sub}" }
                }
            }
            div { class: "fs-card__body", {props.children} }
        }
    }
}

#[cfg(feature = "tui")]
mod tui {
    // TUI fallback stub
}
