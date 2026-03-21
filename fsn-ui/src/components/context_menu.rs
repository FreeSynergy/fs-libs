use dioxus::prelude::*;

/// A single entry in a [`ContextMenu`].
#[derive(Clone, PartialEq)]
pub struct MenuItem {
    /// Display label.
    pub label: String,
    /// Optional icon character or emoji.
    pub icon: Option<String>,
    /// Handler invoked when this item is clicked.
    pub onclick: EventHandler<()>,
}

/// Props for [`ContextMenu`].
#[derive(Props, Clone, PartialEq)]
pub struct ContextMenuProps {
    /// Menu items to display.
    pub items: Vec<MenuItem>,
    /// Whether the context menu is currently visible.
    pub visible: bool,
    /// Handler called when the menu should be closed.
    pub onclose: Option<EventHandler<MouseEvent>>,
    /// Optional extra CSS classes.
    #[props(default)]
    pub class: String,
}

/// A floating context (right-click) menu.
#[component]
pub fn ContextMenu(props: ContextMenuProps) -> Element {
    if !props.visible {
        return rsx! { div { hidden: true } };
    }
    rsx! {
        div {
            class: "fsn-context-menu-overlay",
            role: "presentation",
            onclick: move |e| {
                if let Some(h) = &props.onclose {
                    h.call(e);
                }
            },
        }
        ul {
            class: "fsn-context-menu {props.class}",
            role: "menu",
            for item in &props.items {
                {
                    let handler = item.onclick.clone();
                    rsx! {
                        li { class: "fsn-context-menu__item", role: "menuitem",
                            button {
                                class: "fsn-context-menu__btn",
                                onclick: move |_| handler.call(()),
                                if let Some(icon) = &item.icon {
                                    span { class: "fsn-context-menu__icon", "{icon}" }
                                }
                                span { class: "fsn-context-menu__label", "{item.label}" }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[cfg(feature = "tui")]
mod tui {
    // TUI fallback stub
}
