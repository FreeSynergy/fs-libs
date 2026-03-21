// fs-llm/src/tasks.rs — High-level task functions built on LlmProvider.

use serde::{Deserialize, Serialize};

use crate::error::LlmError;
use crate::provider::LlmProvider;

// ── InterpretedCommand ────────────────────────────────────────────────────────

/// Result of [`interpret_command`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterpretedCommand {
    /// The action name (e.g. `"deploy"`, `"restart"`, `"status"`).
    pub action: String,
    /// Key-value arguments extracted from the command (e.g. `"service" → "matrix"`).
    pub args: std::collections::HashMap<String, String>,
    /// Confidence score in [0.0, 1.0].
    pub confidence: f32,
}

// ── ErrorExplanation ──────────────────────────────────────────────────────────

/// Result of [`explain_error`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorExplanation {
    /// One-sentence summary of what went wrong.
    pub summary: String,
    /// Probable root cause.
    pub cause: String,
    /// Ordered list of suggested fixes.
    pub suggestions: Vec<String>,
}

// ── interpret_command ─────────────────────────────────────────────────────────

/// Interpret a natural-language command into a structured [`InterpretedCommand`].
///
/// The provider is prompted to return JSON with `action`, `args`, and `confidence`.
///
/// # Example
///
/// ```rust,ignore
/// let cmd = interpret_command(&provider, "deploy the matrix service to prod").await?;
/// assert_eq!(cmd.action, "deploy");
/// ```
pub async fn interpret_command(
    provider: &dyn LlmProvider,
    command: &str,
) -> Result<InterpretedCommand, LlmError> {
    let system = r#"You are a command interpreter for a server management system.
Parse the user's natural-language instruction into a JSON object with exactly these fields:
  "action"     : string — the action to perform (deploy, restart, stop, start, status, install, remove, backup, update)
  "args"       : object — key/value string pairs of relevant arguments (service, host, environment, etc.)
  "confidence" : number — your confidence in the parse from 0.0 to 1.0
Reply with ONLY the JSON object, no markdown, no explanation."#;

    let reply = provider
        .ask_with_system(system, command)
        .await?;

    serde_json::from_str::<InterpretedCommand>(reply.trim())
        .map_err(|e| LlmError::task_parse(format!("interpret_command: {e} — raw: {reply}")))
}

// ── summarize_logs ────────────────────────────────────────────────────────────

/// Summarize a block of log lines into a short human-readable description.
///
/// Returns 2–5 sentences describing what happened and any notable events.
pub async fn summarize_logs(
    provider: &dyn LlmProvider,
    logs: &str,
) -> Result<String, LlmError> {
    let system = r#"You are a log analyst for a server management system.
Summarize the provided log excerpt in 2–5 concise sentences.
Focus on: errors, warnings, notable state changes, and performance anomalies.
Do not include raw log lines in your summary. Write plain English."#;

    let prompt = format!("Summarize these logs:\n\n{logs}");
    provider.ask_with_system(system, &prompt).await
}

// ── explain_error ─────────────────────────────────────────────────────────────

/// Explain an error message in plain language with suggested remediation.
///
/// The provider is prompted to return JSON with `summary`, `cause`, and `suggestions`.
pub async fn explain_error(
    provider: &dyn LlmProvider,
    error: &str,
) -> Result<ErrorExplanation, LlmError> {
    let system = r#"You are a systems engineering expert.
Explain the provided error message and suggest fixes.
Reply with ONLY a JSON object with these fields:
  "summary"     : string — one sentence describing what failed
  "cause"       : string — the probable root cause
  "suggestions" : array of strings — ordered list of things to try, most likely first
No markdown, no code fences, just the JSON."#;

    let reply = provider.ask_with_system(system, error).await?;

    serde_json::from_str::<ErrorExplanation>(reply.trim())
        .map_err(|e| LlmError::task_parse(format!("explain_error: {e} — raw: {reply}")))
}

// ── suggest_config ────────────────────────────────────────────────────────────

/// Suggest a corrected or completed TOML config based on partial input.
///
/// `schema_hint` is a brief description of expected fields (not a full schema).
/// Returns a complete, valid TOML string.
pub async fn suggest_config(
    provider: &dyn LlmProvider,
    partial_toml: &str,
    schema_hint: &str,
) -> Result<String, LlmError> {
    let system = format!(
        r#"You are a TOML configuration expert for a server management system.
The user provides a partial TOML config. Complete and correct it.
Expected fields: {schema_hint}
Rules:
- Return ONLY valid TOML, no markdown, no explanation.
- Preserve all values the user already provided.
- Add missing required fields with sensible defaults.
- Fix syntax errors if present."#
    );

    let prompt = format!("Complete this TOML config:\n\n{partial_toml}");
    let reply = provider.ask_with_system(&system, &prompt).await?;

    // Strip optional markdown code fences if the model added them.
    let clean = reply
        .trim()
        .trim_start_matches("```toml")
        .trim_start_matches("```")
        .trim_end_matches("```")
        .trim()
        .to_string();

    Ok(clean)
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::provider::LlmProvider;
    use crate::request::{LlmResponse, Message};
    use async_trait::async_trait;

    struct Stub { reply: &'static str }

    #[async_trait]
    impl LlmProvider for Stub {
        fn name(&self) -> &str { "stub" }
        async fn complete(&self, _: Vec<Message>) -> Result<LlmResponse, LlmError> {
            Ok(LlmResponse::new(self.reply, "stub"))
        }
    }

    #[tokio::test(flavor = "current_thread")]
    async fn interpret_command_parses_json() {
        let stub = Stub {
            reply: r#"{"action":"deploy","args":{"service":"matrix"},"confidence":0.95}"#,
        };
        let cmd = interpret_command(&stub, "deploy matrix").await.unwrap();
        assert_eq!(cmd.action, "deploy");
        assert_eq!(cmd.args.get("service").map(String::as_str), Some("matrix"));
        assert!((cmd.confidence - 0.95).abs() < 0.001);
    }

    #[tokio::test(flavor = "current_thread")]
    async fn explain_error_parses_json() {
        let stub = Stub {
            reply: r#"{"summary":"Port in use","cause":"Another process","suggestions":["Check netstat"]}"#,
        };
        let ex = explain_error(&stub, "bind: address already in use").await.unwrap();
        assert_eq!(ex.summary, "Port in use");
        assert!(!ex.suggestions.is_empty());
    }

    #[tokio::test(flavor = "current_thread")]
    async fn summarize_logs_returns_string() {
        let stub = Stub { reply: "All services started successfully." };
        let summary = summarize_logs(&stub, "INFO service started").await.unwrap();
        assert!(!summary.is_empty());
    }
}
