// fs-components/chat.rs — LlmChat: chat message history + input box.

use dioxus::prelude::*;

// ── ChatRole ──────────────────────────────────────────────────────────────────

/// Speaker role for a `ChatMessage`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChatRole {
    /// Message sent by the human user.
    User,
    /// Response from the language model.
    Assistant,
    /// System-level prompt or context (rendered differently).
    System,
}

impl ChatRole {
    fn label(self) -> &'static str {
        match self {
            Self::User => "You",
            Self::Assistant => "Assistant",
            Self::System => "System",
        }
    }

    fn wrapper_align(self) -> &'static str {
        match self {
            Self::User => "align-self: flex-end;",
            Self::System => "align-self: center; max-width: 100%;",
            Self::Assistant => "align-self: flex-start;",
        }
    }

    fn bubble_style(self) -> &'static str {
        match self {
            Self::User => {
                "background: var(--fs-color-primary, #06b6d4); \
                 color: #000; \
                 border-radius: 14px 14px 4px 14px; \
                 align-self: flex-end;"
            }
            Self::Assistant => {
                "background: var(--fs-color-bg-surface, #1e293b); \
                 color: var(--fs-color-text-primary, #e2e8f0); \
                 border: 1px solid var(--fs-color-border-default, #334155); \
                 border-radius: 14px 14px 14px 4px; \
                 align-self: flex-start;"
            }
            Self::System => {
                "background: rgba(100,116,139,0.15); \
                 color: var(--fs-color-text-secondary, #94a3b8); \
                 border-radius: 8px; \
                 align-self: center; \
                 font-style: italic; font-size: 11px;"
            }
        }
    }
}

// ── ChatMessage ───────────────────────────────────────────────────────────────

/// A single message in an `LlmChat` session.
#[derive(Clone, PartialEq)]
pub struct ChatMessage {
    /// Speaker role.
    pub role: ChatRole,
    /// Text content.
    pub content: String,
    /// Optional human-readable timestamp string.
    pub timestamp: Option<String>,
}

impl ChatMessage {
    /// Construct a user message.
    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: ChatRole::User,
            content: content.into(),
            timestamp: None,
        }
    }

    /// Construct an assistant message.
    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: ChatRole::Assistant,
            content: content.into(),
            timestamp: None,
        }
    }

    /// Construct a system message.
    pub fn system(content: impl Into<String>) -> Self {
        Self {
            role: ChatRole::System,
            content: content.into(),
            timestamp: None,
        }
    }
}

// ── LlmChat ───────────────────────────────────────────────────────────────────

#[derive(Props, Clone, PartialEq)]
pub struct LlmChatProps {
    /// Message history (oldest first).
    pub messages: Vec<ChatMessage>,
    /// Called with the submitted message text when the user sends a message.
    #[props(default)]
    pub on_send: EventHandler<String>,
    /// Shows a typing indicator when the model is responding.
    #[props(default = false)]
    pub loading: bool,
    /// Placeholder text for the input field.
    pub placeholder: Option<String>,
}

/// Full chat interface with scrollable message history and a text input row.
///
/// The component is controlled — the caller owns the `messages` list and
/// handles `on_send` to append new messages and trigger model calls.
///
/// ```no_run
/// LlmChat {
///     messages: messages.read().clone(),
///     loading: *waiting.read(),
///     on_send: move |text| { /* send to model */ },
/// }
/// ```
#[component]
pub fn LlmChat(props: LlmChatProps) -> Element {
    let mut draft = use_signal(String::new);
    let placeholder = props.placeholder.as_deref().unwrap_or("Type a message…");

    rsx! {
        div {
            class: "fs-llm-chat",
            style: "display: flex; flex-direction: column; height: 100%; min-height: 300px; \
                    background: var(--fs-color-bg-canvas, #161b22); \
                    border: 1px solid var(--fs-color-border-default, #334155); \
                    border-radius: 10px; overflow: hidden;",

            // Message history
            div {
                class: "fs-chat-messages",
                style: "flex: 1; overflow-y: auto; padding: 16px; \
                        display: flex; flex-direction: column; gap: 12px;",

                for (idx, msg) in props.messages.iter().enumerate() {
                    {
                        let role_label   = msg.role.label();
                        let bubble_style = msg.role.bubble_style();
                        let content      = msg.content.clone();
                        let ts           = msg.timestamp.clone();

                        let wrapper_align = msg.role.wrapper_align();
                        let label_align = if msg.role == ChatRole::User {
                            "flex-end"
                        } else {
                            "flex-start"
                        };

                        rsx! {
                            div {
                                key: "{idx}",
                                class: "fs-chat-msg",
                                style: "display: flex; flex-direction: column; gap: 4px; \
                                        max-width: 78%; {wrapper_align}",

                                // Role label
                                span {
                                    style: "font-size: 10px; font-weight: 600; \
                                            letter-spacing: 0.06em; \
                                            color: var(--fs-color-text-muted, #64748b); \
                                            text-transform: uppercase; \
                                            align-self: {label_align};",
                                    "{role_label}"
                                }

                                // Bubble
                                div {
                                    style: "padding: 10px 14px; font-size: 13px; \
                                            line-height: 1.6; white-space: pre-wrap; \
                                            word-break: break-word; {bubble_style}",
                                    "{content}"
                                }

                                // Timestamp
                                if let Some(t) = &ts {
                                    span {
                                        style: "font-size: 10px; \
                                                color: var(--fs-color-text-muted, #64748b); \
                                                align-self: {label_align};",
                                        "{t}"
                                    }
                                }
                            }
                        }
                    }
                }

                // Typing indicator
                if props.loading {
                    div {
                        class: "fs-chat-typing",
                        style: "display: flex; gap: 4px; align-items: center; \
                                padding: 10px 14px; border-radius: 14px 14px 14px 4px; \
                                background: var(--fs-color-bg-surface, #1e293b); \
                                border: 1px solid var(--fs-color-border-default, #334155); \
                                align-self: flex-start;",
                        span {
                            style: "width: 6px; height: 6px; border-radius: 50%; \
                                    background: var(--fs-color-text-muted, #64748b); \
                                    animation: fs-bounce 1.2s ease-in-out infinite;",
                        }
                        span {
                            style: "width: 6px; height: 6px; border-radius: 50%; \
                                    background: var(--fs-color-text-muted, #64748b); \
                                    animation: fs-bounce 1.2s ease-in-out 0.2s infinite;",
                        }
                        span {
                            style: "width: 6px; height: 6px; border-radius: 50%; \
                                    background: var(--fs-color-text-muted, #64748b); \
                                    animation: fs-bounce 1.2s ease-in-out 0.4s infinite;",
                        }
                    }
                }
            }

            // Keyframes for typing indicator
            style {
                "@keyframes fs-bounce {{ \
                    0%, 60%, 100% {{ transform: translateY(0); }} \
                    30% {{ transform: translateY(-6px); }} \
                }}"
            }

            // Input row
            div {
                class: "fs-chat-input-row",
                style: "display: flex; gap: 8px; padding: 12px 14px; \
                        border-top: 1px solid var(--fs-color-border-default, #334155); \
                        background: var(--fs-color-bg-surface, #1e293b);",

                textarea {
                    rows: 1_u32,
                    placeholder: "{placeholder}",
                    value: "{draft}",
                    style: "flex: 1; resize: none; background: none; border: none; \
                            outline: none; font-size: 13px; font-family: inherit; \
                            color: var(--fs-color-text-primary, #e2e8f0); \
                            padding: 6px 0; line-height: 1.5;",
                    oninput: move |e| {
                        *draft.write() = e.value();
                    },
                    onkeydown: {
                        let trigger = props.on_send.clone();
                        move |e: KeyboardEvent| {
                            // Ctrl+Enter or Shift+Enter adds a newline; plain Enter sends.
                            if e.key() == Key::Enter
                                && !e.modifiers().ctrl()
                                && !e.modifiers().shift()
                            {
                                e.prevent_default();
                                let text = draft.read().trim().to_string();
                                if !text.is_empty() {
                                    trigger.call(text);
                                    *draft.write() = String::new();
                                }
                            }
                        }
                    },
                }

                button {
                    aria_label: "Send message",
                    disabled: props.loading || draft.read().trim().is_empty(),
                    style: "padding: 6px 14px; border-radius: 6px; border: none; \
                            cursor: pointer; font-size: 13px; font-weight: 500; \
                            font-family: inherit; \
                            background: var(--fs-color-primary, #06b6d4); \
                            color: #000; \
                            transition: opacity 0.15s;",
                    onclick: {
                        let trigger = props.on_send.clone();
                        move |_| {
                            let text = draft.read().trim().to_string();
                            if !text.is_empty() {
                                trigger.call(text);
                                *draft.write() = String::new();
                            }
                        }
                    },
                    "Send"
                }
            }
        }
    }
}
