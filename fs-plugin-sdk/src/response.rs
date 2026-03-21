// PluginResponse — JSON payload the plugin writes to its stdout.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

fn protocol_v1() -> u32 {
    1
}

fn default_mode() -> u32 {
    0o644
}

// ── PluginResponse ────────────────────────────────────────────────────────────

/// JSON payload the plugin writes to its stdout.
///
/// Core reads this, writes declared files, and runs declared shell commands
/// in order.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PluginResponse {
    /// Protocol version — Core rejects if it doesn't match.
    #[serde(default = "protocol_v1")]
    pub protocol: u32,

    /// Log lines to emit (Core prefixes with the module name).
    #[serde(default)]
    pub logs: Vec<LogLine>,

    /// Files to write to disk.
    #[serde(default)]
    pub files: Vec<OutputFile>,

    /// Shell commands to run after files are written (in order, must succeed).
    #[serde(default)]
    pub commands: Vec<ShellCommand>,

    /// Non-empty = plugin reported an error; Core aborts with this message.
    #[serde(default)]
    pub error: String,
}

impl PluginResponse {
    /// Construct an error response with the given message.
    pub fn err(message: impl Into<String>) -> Self {
        Self { error: message.into(), ..Default::default() }
    }

    /// `true` when the plugin reported an error (i.e. `error` is non-empty).
    pub fn has_error(&self) -> bool {
        !self.error.is_empty()
    }
}

// ── OutputFile ────────────────────────────────────────────────────────────────

/// A file the plugin wants Core to write.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputFile {
    /// Absolute destination path.
    pub dest: String,

    /// File contents (UTF-8 text).
    pub content: String,

    /// Unix permission bits (e.g. `0o644`). Defaults to `0o644` if absent.
    #[serde(default = "default_mode")]
    pub mode: u32,
}

// ── ShellCommand ──────────────────────────────────────────────────────────────

/// A shell command the plugin wants Core to run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShellCommand {
    /// The command string (passed verbatim to `sh -c`).
    pub cmd: String,

    /// Working directory (absolute path). Defaults to process cwd if absent.
    pub cwd: Option<String>,

    /// Environment variables for this command (merged with process env).
    #[serde(default)]
    pub env: HashMap<String, String>,
}

// ── LogLine ───────────────────────────────────────────────────────────────────

/// A log line emitted by the plugin.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogLine {
    /// Severity of this log entry.
    pub level: LogLevel,

    /// Human-readable log message.
    pub message: String,
}

/// Severity level for plugin log lines.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    /// Informational message.
    #[default]
    Info,

    /// Warning — something unexpected but non-fatal.
    Warn,

    /// Error — plugin encountered a problem.
    Error,
}
