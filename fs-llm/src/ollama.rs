// fs-llm/src/ollama.rs — Ollama provider (feature: ollama).

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tracing::debug;

use crate::error::LlmError;
use crate::provider::LlmProvider;
use crate::request::{LlmResponse, Message, Role, TokenUsage};

// ── Wire types ────────────────────────────────────────────────────────────────

#[derive(Serialize)]
struct OllamaRequest<'a> {
    model: &'a str,
    messages: Vec<OllamaMessage<'a>>,
    stream: bool,
}

#[derive(Serialize)]
struct OllamaMessage<'a> {
    role: &'a str,
    content: &'a str,
}

#[derive(Deserialize)]
struct OllamaResponse {
    message: OllamaMessageOut,
    model: String,
    #[serde(default)]
    prompt_eval_count: Option<u32>,
    #[serde(default)]
    eval_count: Option<u32>,
}

#[derive(Deserialize)]
struct OllamaMessageOut {
    content: String,
}

// ── OllamaProvider ────────────────────────────────────────────────────────────

/// LLM provider for locally-running [Ollama](https://ollama.ai) instances.
///
/// # Example
///
/// ```rust,ignore
/// use fs_llm::OllamaProvider;
///
/// let provider = OllamaProvider::new("http://localhost:11434", "llama3");
/// let reply = provider.ask("Explain Rust lifetimes in one sentence.").await?;
/// ```
pub struct OllamaProvider {
    base_url: String,
    model: String,
    client: reqwest::Client,
}

impl OllamaProvider {
    /// Create a provider pointing at `base_url` (e.g. `"http://localhost:11434"`)
    /// using `model` (e.g. `"llama3"`, `"mistral"`).
    pub fn new(base_url: impl Into<String>, model: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into().trim_end_matches('/').to_string(),
            model: model.into(),
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl LlmProvider for OllamaProvider {
    fn name(&self) -> &str {
        "ollama"
    }

    async fn complete(&self, messages: Vec<Message>) -> Result<LlmResponse, LlmError> {
        let wire_msgs: Vec<OllamaMessage<'_>> = messages
            .iter()
            .map(|m| OllamaMessage {
                role: match m.role {
                    Role::System    => "system",
                    Role::User      => "user",
                    Role::Assistant => "assistant",
                },
                content: &m.content,
            })
            .collect();

        let req = OllamaRequest { model: &self.model, messages: wire_msgs, stream: false };
        let url = format!("{}/api/chat", self.base_url);

        debug!(url = %url, model = %self.model, "ollama request");

        let resp = self
            .client
            .post(&url)
            .json(&req)
            .send()
            .await
            .map_err(|e| LlmError::network(e.to_string()))?;

        let status = resp.status().as_u16();
        if !resp.status().is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(LlmError::provider(status, body));
        }

        let body: OllamaResponse = resp
            .json()
            .await
            .map_err(|e| LlmError::serialization(e.to_string()))?;

        let usage = match (body.prompt_eval_count, body.eval_count) {
            (Some(p), Some(c)) => Some(TokenUsage { prompt_tokens: p, completion_tokens: c }),
            _ => None,
        };

        Ok(LlmResponse {
            content: body.message.content,
            model: body.model,
            usage,
        })
    }
}
