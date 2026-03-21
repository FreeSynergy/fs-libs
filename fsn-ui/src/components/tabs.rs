use dioxus::prelude::*;

/// A single tab descriptor for [`Tabs`].
#[derive(Clone, PartialEq)]
pub struct TabItem {
    /// Unique identifier for the tab.
    pub id: String,
    /// Human-readable tab label.
    pub label: String,
}

/// Props for [`Tabs`].
#[derive(Props, Clone, PartialEq)]
pub struct TabsProps {
    /// All available tabs.
    pub tabs: Vec<TabItem>,
    /// Currently active tab id.
    pub active_tab: String,
    /// Handler called with the id of the newly selected tab.
    pub onchange: Option<EventHandler<String>>,
    /// Optional extra CSS classes.
    #[props(default)]
    pub class: String,
}

/// A horizontal tab bar.
#[component]
pub fn Tabs(props: TabsProps) -> Element {
    rsx! {
        div { class: "fsn-tabs {props.class}", role: "tablist",
            for tab in &props.tabs {
                {
                    let tab_id = tab.id.clone();
                    let is_active = tab.id == props.active_tab;
                    let active_class = if is_active { "fsn-tabs__tab--active" } else { "" };
                    rsx! {
                        button {
                            class: "fsn-tabs__tab {active_class}",
                            role: "tab",
                            aria_selected: if is_active { "true" } else { "false" },
                            onclick: move |_| {
                                if let Some(h) = &props.onchange {
                                    h.call(tab_id.clone());
                                }
                            },
                            "{tab.label}"
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
