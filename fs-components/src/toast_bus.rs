// fs-components/toast_bus.rs — Renderer-agnostic message buses.
//
// ToastBus and ErrorBus are broadcast channels that work from any context:
// Dioxus components, CLI handlers, background tokio tasks.
// Senders are cheap to clone; receivers are created on demand via `.subscribe()`.

use tokio::sync::broadcast;

// ── ToastLevel ────────────────────────────────────────────────────────────────

/// Severity of a toast notification.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToastLevel {
    Info,
    Success,
    Warning,
    Error,
}

impl ToastLevel {
    /// CSS `border-left` declaration for this severity level.
    pub fn border_css(self) -> &'static str {
        match self {
            Self::Info    => "border-left: 3px solid #06b6d4;",
            Self::Success => "border-left: 3px solid #22c55e;",
            Self::Warning => "border-left: 3px solid #f59e0b;",
            Self::Error   => "border-left: 3px solid #ef4444;",
        }
    }

    /// Unicode icon character representing this severity level.
    pub fn icon(self) -> &'static str {
        match self {
            Self::Info    => "ℹ",
            Self::Success => "✓",
            Self::Warning => "⚠",
            Self::Error   => "✗",
        }
    }
}

// ── ToastMessage ──────────────────────────────────────────────────────────────

/// A single toast notification message.
#[derive(Debug, Clone)]
pub struct ToastMessage {
    pub level:   ToastLevel,
    pub title:   String,
    /// Optional body text shown beneath the title.
    pub body:    Option<String>,
    /// Auto-dismiss after this many milliseconds. `None` = sticky.
    pub timeout: Option<u32>,
}

impl ToastMessage {
    /// Convenience constructor for simple info toasts.
    pub fn info(title: impl Into<String>) -> Self {
        Self { level: ToastLevel::Info, title: title.into(), body: None, timeout: Some(3000) }
    }

    /// Convenience constructor for success toasts.
    pub fn success(title: impl Into<String>) -> Self {
        Self { level: ToastLevel::Success, title: title.into(), body: None, timeout: Some(3000) }
    }

    /// Convenience constructor for warning toasts.
    pub fn warning(title: impl Into<String>) -> Self {
        Self { level: ToastLevel::Warning, title: title.into(), body: None, timeout: Some(5000) }
    }

    /// Convenience constructor for sticky error toasts.
    pub fn error(title: impl Into<String>, body: impl Into<String>) -> Self {
        Self {
            level:   ToastLevel::Error,
            title:   title.into(),
            body:    Some(body.into()),
            timeout: None,
        }
    }
}

// ── ToastBus ──────────────────────────────────────────────────────────────────

/// Global broadcast bus for toast notifications.
///
/// Call [`ToastBus::sender`] once at startup to obtain a `Sender<ToastMessage>`,
/// then clone it wherever you need to emit toasts.
///
/// ```no_run
/// use fs_components::{ToastBus, ToastMessage};
/// let tx = ToastBus::sender();
/// tx.send(ToastMessage::success("Deployed!")).ok();
/// ```
pub struct ToastBus;

impl ToastBus {
    const CAPACITY: usize = 64;

    /// Returns a new broadcast sender for the toast channel.
    ///
    /// Each call creates an independent channel. Hold the sender somewhere static
    /// (e.g. a `OnceLock` or Dioxus `use_context`) and share it by cloning.
    pub fn sender() -> broadcast::Sender<ToastMessage> {
        broadcast::channel(Self::CAPACITY).0
    }
}

// ── ErrorMessage ──────────────────────────────────────────────────────────────

/// A structured error event, suitable for logging and display.
#[derive(Debug, Clone)]
pub struct ErrorMessage {
    /// Short human-readable summary.
    pub title:   String,
    /// `Debug` or structured representation of the underlying error.
    pub detail:  String,
    /// Optional source location hint.
    pub context: Option<String>,
}

impl ErrorMessage {
    /// Build an `ErrorMessage` from any `std::error::Error` implementation.
    pub fn from_err(title: impl Into<String>, err: &dyn std::error::Error) -> Self {
        Self {
            title:   title.into(),
            detail:  format!("{err:#}"),
            context: None,
        }
    }

    /// Attach a context hint (e.g. `"deploy::reconcile"`).
    pub fn with_context(mut self, ctx: impl Into<String>) -> Self {
        self.context = Some(ctx.into());
        self
    }
}

// ── ErrorBus ─────────────────────────────────────────────────────────────────

/// Global broadcast bus for structured error events.
///
/// Errors can be sent from any context (background tasks, CLI handlers) and
/// consumed by the UI to display error panels or log entries.
pub struct ErrorBus;

impl ErrorBus {
    const CAPACITY: usize = 32;

    /// Returns a new broadcast sender for the error channel.
    pub fn sender() -> broadcast::Sender<ErrorMessage> {
        broadcast::channel(Self::CAPACITY).0
    }
}
