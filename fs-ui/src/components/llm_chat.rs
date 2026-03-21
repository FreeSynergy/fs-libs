use dioxus::prelude::*;

/// A single message in the [`LlmChat`] history.
#[derive(Clone, PartialEq)]
pub struct ChatMessage {
    /// `"user"` or `"assistant"`.
    pub role: String,
    /// Message text content.
    pub content: String,
}

/// Props for [`LlmChat`].
#[derive(Props, Clone, PartialEq)]
pub struct LlmChatProps {
    /// All messages in the current conversation.
    pub messages: Vec<ChatMessage>,
    /// Handler called with the user's input when they submit a message.
    pub onsubmit: EventHandler<String>,
    /// Whether the assistant is currently generating a response.
    #[props(default = false)]
    pub loading: bool,
    /// Optional extra CSS classes.
    #[props(default)]
    pub class: String,
}

/// A chat interface for LLM conversations.
#[component]
pub fn LlmChat(props: LlmChatProps) -> Element {
    let mut draft = use_signal(String::new);

    rsx! {
        div { class: "fs-llm-chat {props.class}",
            div { class: "fs-llm-chat__messages", role: "log", aria_live: "polite",
                for msg in &props.messages {
                    {
                        let role_class = format!("fs-llm-chat__msg--{}", msg.role);
                        rsx! {
                            div { class: "fs-llm-chat__msg {role_class}",
                                span { class: "fs-llm-chat__role", "{msg.role}" }
                                p { class: "fs-llm-chat__content", "{msg.content}" }
                            }
                        }
                    }
                }
                if props.loading {
                    div { class: "fs-llm-chat__typing",
                        span { class: "fs-spinner fs-spinner--sm", role: "status", aria_label: "Thinking…" }
                        span { "Thinking…" }
                    }
                }
            }
            form {
                class: "fs-llm-chat__input-row",
                onsubmit: move |e| {
                    e.prevent_default();
                    let text = draft.read().trim().to_string();
                    if !text.is_empty() {
                        props.onsubmit.call(text);
                        draft.set(String::new());
                    }
                },
                textarea {
                    class: "fs-llm-chat__textarea",
                    value: "{draft}",
                    placeholder: "Type a message…",
                    rows: "3",
                    disabled: props.loading,
                    oninput: move |e| draft.set(e.value()),
                }
                button {
                    class: "fs-button fs-button--primary",
                    r#type: "submit",
                    disabled: props.loading,
                    "Send"
                }
            }
        }
    }
}

#[cfg(feature = "tui")]
mod tui {
    // TUI fallback stub
}
