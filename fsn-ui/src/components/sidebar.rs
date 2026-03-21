use dioxus::prelude::*;

/// A single item in the [`Sidebar`].
#[derive(Clone, PartialEq)]
pub struct SidebarItem {
    /// Unique identifier for the item.
    pub id: String,
    /// Display label.
    pub label: String,
    /// Icon character or emoji.
    pub icon: String,
    /// Whether this item is currently active/selected.
    pub active: bool,
}

/// Props for [`Sidebar`].
#[derive(Props, Clone, PartialEq)]
pub struct SidebarProps {
    /// Navigation items.
    pub items: Vec<SidebarItem>,
    /// Handler called with the `id` of the selected item.
    pub onselect: Option<EventHandler<String>>,
    /// Optional extra CSS classes.
    #[props(default)]
    pub class: String,
}

/// A vertical navigation sidebar.
#[component]
pub fn Sidebar(props: SidebarProps) -> Element {
    rsx! {
        nav {
            class: "fsn-sidebar {props.class}",
            aria_label: "Main navigation",
            ul { class: "fsn-sidebar__list",
                for item in &props.items {
                    {
                        let item_id = item.id.clone();
                        let active_class = if item.active { "fsn-sidebar__item--active" } else { "" };
                        rsx! {
                            li {
                                class: "fsn-sidebar__item {active_class}",
                                button {
                                    class: "fsn-sidebar__btn",
                                    aria_current: if item.active { "page" } else { "false" },
                                    onclick: move |_| {
                                        if let Some(h) = &props.onselect {
                                            h.call(item_id.clone());
                                        }
                                    },
                                    span {
                                        class: "fsn-sidebar__icon",
                                        if item.icon.trim_start().starts_with("<svg") {
                                            span { dangerous_inner_html: "{item.icon}" }
                                        } else {
                                            "{item.icon}"
                                        }
                                    }
                                    span { class: "fsn-sidebar__label", "{item.label}" }
                                }
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
