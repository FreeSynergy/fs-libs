// fs-llm/src/openai_compat.rs — OpenAI-compatible provider (feature: openai-compat).
//
// Works with: OpenAI, Groq, Together, Mistral, local servers like LM Studio.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tracing::debug;

use crate::error::LlmError;
use crate::provider::LlmProvider;
use crate::request::{LlmResponse, Message, Role, TokenUsage};

// ── Wire types ────────────────────────────────────────────────────────────────

#[derive(Serialize)]
struct OaiRequest<'a> {
    model: &'a str,
    messages: Vec<OaiMessage<'a>>,
}

#[derive(Serialize)]
struct OaiMessage<'a> {
    role: &'a str,
    content: &'a str,
}

#[derive(Deserialize)]
struct OaiResponse {
    model: String,
    choices: Vec<OaiChoice>,
    #[serde(default)]
    usage: Option<OaiUsage>,
}

#[derive(Deserialize)]
struct OaiChoice {
    message: OaiMessageOut,
}

#[derive(Deserialize)]
struct OaiMessageOut {
    content: String,
}

#[derive(Deserialize)]
struct OaiUsage {
    prompt_tokens: u32,
    completion_tokens: u32,
}

// ── OpenAiCompatProvider ──────────────────────────────────────────────────────

/// LLM provider compatible with the OpenAI Chat Completions API.
///
/// Works with any server that exposes `POST /v1/chat/completions` in OpenAI
/// format — including OpenAI itself, Groq, Together AI, Mistral, and LM Studio.
///
/// # Example
///
/// ```rust,ignore
/// use fs_llm::OpenAiCompatProvider;
///
/// // Groq
/// let provider = OpenAiCompatProvider::new(
///     "https://api.groq.com/openai",
///     "llama3-70b-8192",
///     Some("your-groq-api-key"),
/// );
///
/// // Local LM Studio
/// let local = OpenAiCompatProvider::new("http://localhost:1234", "local-model", None);
/// ```
pub struct OpenAiCompatProvider {
    base_url: String,
    model: String,
    api_key: Option<String>,
    client: reqwest::Client,
}

impl OpenAiCompatProvider {
    /// Create a provider.
    ///
    /// - `base_url` — server root, e.g. `"https://api.openai.com"` (no trailing slash)
    /// - `model`    — model identifier, e.g. `"gpt-4o"`, `"llama3-70b-8192"`
    /// - `api_key`  — bearer token (pass `None` for unauthenticated local servers)
    pub fn new(
        base_url: impl Into<String>,
        model: impl Into<String>,
        api_key: Option<impl Into<String>>,
    ) -> Self {
        Self {
            base_url: base_url.into().trim_end_matches('/').to_string(),
            model: model.into(),
            api_key: api_key.map(Into::into),
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl LlmProvider for OpenAiCompatProvider {
    fn name(&self) -> &str {
        "openai-compat"
    }

    async fn complete(&self, messages: Vec<Message>) -> Result<LlmResponse, LlmError> {
        let wire_msgs: Vec<OaiMessage<'_>> = messages
            .iter()
            .map(|m| OaiMessage {
                role: match m.role {
                    Role::System    => "system",
                    Role::User      => "user",
                    Role::Assistant => "assistant",
                },
                content: &m.content,
            })
            .collect();

        let req = OaiRequest { model: &self.model, messages: wire_msgs };
        let url = format!("{}/v1/chat/completions", self.base_url);

        debug!(url = %url, model = %self.model, "openai-compat request");

        let mut builder = self.client.post(&url).json(&req);
        if let Some(key) = &self.api_key {
            builder = builder.bearer_auth(key);
        }

        let resp = builder.send().await.map_err(|e| LlmError::network(e.to_string()))?;

        let status = resp.status().as_u16();
        if !resp.status().is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(LlmError::provider(status, body));
        }

        let body: OaiResponse = resp
            .json()
            .await
            .map_err(|e| LlmError::serialization(e.to_string()))?;

        let content = body
            .choices
            .into_iter()
            .next()
            .map(|c| c.message.content)
            .ok_or(LlmError::EmptyResponse)?;

        let usage = body.usage.map(|u| TokenUsage {
            prompt_tokens: u.prompt_tokens,
            completion_tokens: u.completion_tokens,
        });

        Ok(LlmResponse { content, model: body.model, usage })
    }
}
