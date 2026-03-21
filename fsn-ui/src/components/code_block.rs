use dioxus::prelude::*;

/// Props for [`CodeBlock`].
#[derive(Props, Clone, PartialEq)]
pub struct CodeBlockProps {
    /// The source code to display.
    pub code: String,
    /// Language identifier for syntax hint (e.g. `"rust"`, `"toml"`).
    #[props(default)]
    pub language: String,
    /// Optional filename shown in the header.
    pub filename: Option<String>,
    /// Optional extra CSS classes.
    #[props(default)]
    pub class: String,
}

/// A syntax-highlighted code block with an optional filename header.
#[component]
pub fn CodeBlock(props: CodeBlockProps) -> Element {
    rsx! {
        div { class: "fsn-code-block {props.class}",
            if props.filename.is_some() || !props.language.is_empty() {
                div { class: "fsn-code-block__header",
                    if let Some(filename) = &props.filename {
                        span { class: "fsn-code-block__filename", "{filename}" }
                    }
                    if !props.language.is_empty() {
                        span { class: "fsn-code-block__lang", "{props.language}" }
                    }
                }
            }
            pre { class: "fsn-code-block__pre",
                code {
                    class: "fsn-code-block__code language-{props.language}",
                    "{props.code}"
                }
            }
        }
    }
}

#[cfg(feature = "tui")]
mod tui {
    // TUI fallback stub
}
