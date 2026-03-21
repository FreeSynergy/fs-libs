// fsn-llm/src/claude.rs — Anthropic Claude provider (feature: claude).

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tracing::debug;

use crate::error::LlmError;
use crate::provider::LlmProvider;
use crate::request::{LlmResponse, Message, Role, TokenUsage};

const ANTHROPIC_API_URL: &str = "https://api.anthropic.com/v1/messages";
const ANTHROPIC_VERSION: &str = "2023-06-01";

// ── Wire types ────────────────────────────────────────────────────────────────

#[derive(Serialize)]
struct ClaudeRequest<'a> {
    model: &'a str,
    max_tokens: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    system: Option<&'a str>,
    messages: Vec<ClaudeMessage<'a>>,
}

#[derive(Serialize)]
struct ClaudeMessage<'a> {
    role: &'a str,
    content: &'a str,
}

#[derive(Deserialize)]
struct ClaudeResponse {
    model: String,
    content: Vec<ClaudeContent>,
    #[serde(default)]
    usage: Option<ClaudeUsage>,
}

#[derive(Deserialize)]
struct ClaudeContent {
    #[serde(rename = "type")]
    kind: String,
    text: Option<String>,
}

#[derive(Deserialize)]
struct ClaudeUsage {
    input_tokens: u32,
    output_tokens: u32,
}

// ── ClaudeProvider ────────────────────────────────────────────────────────────

/// LLM provider for the [Anthropic Claude](https://anthropic.com) API.
///
/// Requires feature `claude`.
///
/// # Example
///
/// ```rust,ignore
/// use fsn_llm::ClaudeProvider;
///
/// let provider = ClaudeProvider::new("sk-ant-...", "claude-sonnet-4-6");
/// let reply = provider.ask("What is CRDT?").await?;
/// ```
pub struct ClaudeProvider {
    api_key: String,
    model: String,
    max_tokens: u32,
    client: reqwest::Client,
}

impl ClaudeProvider {
    /// Create a provider with the given API key and model.
    ///
    /// `max_tokens` defaults to 4096 — use [`ClaudeProvider::with_max_tokens`]
    /// to override.
    pub fn new(api_key: impl Into<String>, model: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            model: model.into(),
            max_tokens: 4096,
            client: reqwest::Client::new(),
        }
    }

    /// Set the maximum number of output tokens.
    pub fn with_max_tokens(mut self, max_tokens: u32) -> Self {
        self.max_tokens = max_tokens;
        self
    }
}

#[async_trait]
impl LlmProvider for ClaudeProvider {
    fn name(&self) -> &str {
        "claude"
    }

    async fn complete(&self, messages: Vec<Message>) -> Result<LlmResponse, LlmError> {
        // Anthropic API separates the system prompt from the message list.
        let system = messages
            .iter()
            .find(|m| m.role == Role::System)
            .map(|m| m.content.as_str());

        let chat_msgs: Vec<ClaudeMessage<'_>> = messages
            .iter()
            .filter(|m| m.role != Role::System)
            .map(|m| ClaudeMessage {
                role: match m.role {
                    Role::User      => "user",
                    Role::Assistant => "assistant",
                    Role::System    => unreachable!(),
                },
                content: &m.content,
            })
            .collect();

        let req = ClaudeRequest {
            model: &self.model,
            max_tokens: self.max_tokens,
            system,
            messages: chat_msgs,
        };

        debug!(model = %self.model, "claude request");

        let resp = self
            .client
            .post(ANTHROPIC_API_URL)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", ANTHROPIC_VERSION)
            .header("content-type", "application/json")
            .json(&req)
            .send()
            .await
            .map_err(|e| LlmError::network(e.to_string()))?;

        let status = resp.status().as_u16();
        if !resp.status().is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(LlmError::provider(status, body));
        }

        let body: ClaudeResponse = resp
            .json()
            .await
            .map_err(|e| LlmError::serialization(e.to_string()))?;

        let content = body
            .content
            .into_iter()
            .find(|c| c.kind == "text")
            .and_then(|c| c.text)
            .ok_or(LlmError::EmptyResponse)?;

        let usage = body.usage.map(|u| TokenUsage {
            prompt_tokens: u.input_tokens,
            completion_tokens: u.output_tokens,
        });

        Ok(LlmResponse { content, model: body.model, usage })
    }
}
