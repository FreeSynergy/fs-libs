use dioxus::prelude::*;

/// An app entry displayed in the [`AppLauncher`] grid.
#[derive(Clone, PartialEq)]
pub struct AppEntry {
    /// Unique app identifier.
    pub id: String,
    /// Human-readable app name.
    pub name: String,
    /// Icon character, emoji, or URL.
    pub icon: String,
    /// App URL.
    pub url: String,
    /// Status indicator text (e.g. "running", "stopped").
    pub status: String,
}

/// Props for [`AppLauncher`].
#[derive(Props, Clone, PartialEq)]
pub struct AppLauncherProps {
    /// Available apps.
    pub apps: Vec<AppEntry>,
    /// Handler called with the `id` of the launched app.
    pub onlaunch: Option<EventHandler<String>>,
    /// Optional extra CSS classes.
    #[props(default)]
    pub class: String,
}

/// A grid-based application launcher.
#[component]
pub fn AppLauncher(props: AppLauncherProps) -> Element {
    rsx! {
        div { class: "fsn-app-launcher {props.class}", role: "list",
            for app in &props.apps {
                {
                    let app_id = app.id.clone();
                    let status_class = format!("fsn-app-launcher__status--{}", app.status);
                    rsx! {
                        div {
                            class: "fsn-app-launcher__item",
                            role: "listitem",
                            button {
                                class: "fsn-app-launcher__btn",
                                title: "{app.name}",
                                onclick: move |_| {
                                    if let Some(h) = &props.onlaunch {
                                        h.call(app_id.clone());
                                    }
                                },
                                span { class: "fsn-app-launcher__icon", "{app.icon}" }
                                span { class: "fsn-app-launcher__name", "{app.name}" }
                                span {
                                    class: "fsn-app-launcher__status {status_class}",
                                    "{app.status}"
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
