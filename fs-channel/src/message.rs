// fs-channel/src/message.rs — Channel message types.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// ── MessageKind ───────────────────────────────────────────────────────────────

/// The rendering style for an outgoing message.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MessageKind {
    /// Plain text.
    Text,
    /// Markdown-formatted text (rendered where supported).
    Markdown,
    /// A notice — typically displayed differently from regular messages.
    Notice,
    /// A code block with optional language tag.
    Code { language: Option<String> },
}

impl MessageKind {
    /// Returns `true` if this kind should be sent as rich/formatted text.
    pub fn is_rich(&self) -> bool {
        matches!(self, Self::Markdown | Self::Code { .. })
    }

    /// Renders `body` according to this kind (wraps code in fences, etc.).
    pub fn render_body<'a>(&self, body: &'a str) -> std::borrow::Cow<'a, str> {
        match self {
            Self::Code {
                language: Some(lang),
            } => std::borrow::Cow::Owned(format!("```{lang}\n{body}\n```")),
            Self::Code { language: None } => std::borrow::Cow::Owned(format!("```\n{body}\n```")),
            _ => std::borrow::Cow::Borrowed(body),
        }
    }
}

// ── Attachment ────────────────────────────────────────────────────────────────

/// A file attached to a message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attachment {
    /// Original filename.
    pub filename: String,
    /// MIME type, e.g. `"image/png"` or `"text/plain"`.
    pub mime_type: String,
    /// Raw file bytes.
    pub data: Vec<u8>,
}

// ── ChannelMessage ────────────────────────────────────────────────────────────

/// A message to send through a [`Channel`](crate::channel::Channel).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelMessage {
    /// How the body should be rendered.
    pub kind: MessageKind,
    /// Text content of the message.
    pub body: String,
    /// Optional file attachments.
    pub attachments: Vec<Attachment>,
}

impl ChannelMessage {
    /// Create a plain-text message.
    pub fn text(body: impl Into<String>) -> Self {
        Self {
            kind: MessageKind::Text,
            body: body.into(),
            attachments: Vec::new(),
        }
    }

    /// Create a Markdown message.
    pub fn markdown(body: impl Into<String>) -> Self {
        Self {
            kind: MessageKind::Markdown,
            body: body.into(),
            attachments: Vec::new(),
        }
    }

    /// Create a notice message.
    pub fn notice(body: impl Into<String>) -> Self {
        Self {
            kind: MessageKind::Notice,
            body: body.into(),
            attachments: Vec::new(),
        }
    }

    /// Create a code message.
    pub fn code(body: impl Into<String>, language: Option<impl Into<String>>) -> Self {
        Self {
            kind: MessageKind::Code {
                language: language.map(Into::into),
            },
            body: body.into(),
            attachments: Vec::new(),
        }
    }

    /// Attach a file to this message.
    pub fn with_attachment(mut self, attachment: Attachment) -> Self {
        self.attachments.push(attachment);
        self
    }

    /// Rendered body according to this message's kind.
    pub fn rendered_body(&self) -> std::borrow::Cow<'_, str> {
        self.kind.render_body(&self.body)
    }
}

// ── IncomingMessage ───────────────────────────────────────────────────────────

/// A message received through a [`Channel`](crate::channel::Channel).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncomingMessage {
    /// The room, chat, or channel ID the message came from.
    pub room_id: String,
    /// Identifier of the sender (user ID, username, etc.).
    pub sender: String,
    /// Plain-text body of the message.
    pub body: String,
    /// UTC timestamp when the message was sent.
    pub timestamp: DateTime<Utc>,
}
