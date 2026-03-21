// fsn-llm/src/provider.rs — Core LlmProvider trait.

use async_trait::async_trait;

use crate::error::LlmError;
use crate::request::{LlmResponse, Message};

// ── LlmProvider ───────────────────────────────────────────────────────────────

/// Abstraction over different LLM backends.
///
/// Implement this trait to add a new provider (Ollama, Claude, OpenAI-compat, …).
/// Task functions in [`crate::tasks`] accept any `&dyn LlmProvider`.
///
/// # Example
///
/// ```rust,ignore
/// use fsn_llm::{LlmProvider, Message, LlmResponse, LlmError};
/// use async_trait::async_trait;
///
/// struct Echo;
///
/// #[async_trait]
/// impl LlmProvider for Echo {
///     fn name(&self) -> &str { "echo" }
///
///     async fn complete(&self, messages: Vec<Message>) -> Result<LlmResponse, LlmError> {
///         let last = messages.last().map(|m| m.content.as_str()).unwrap_or("");
///         Ok(LlmResponse::new(last, "echo"))
///     }
/// }
/// ```
#[async_trait]
pub trait LlmProvider: Send + Sync {
    /// Short identifier for this provider, e.g. `"ollama"`, `"claude"`.
    fn name(&self) -> &str;

    /// Send `messages` and return the assistant's reply.
    async fn complete(&self, messages: Vec<Message>) -> Result<LlmResponse, LlmError>;

    /// Convenience: send a single user message and return the reply text.
    async fn ask(&self, prompt: &str) -> Result<String, LlmError> {
        let msg = Message::user(prompt);
        Ok(self.complete(vec![msg]).await?.content)
    }

    /// Convenience: send system + user message.
    async fn ask_with_system(
        &self,
        system: &str,
        prompt: &str,
    ) -> Result<String, LlmError> {
        let msgs = vec![Message::system(system), Message::user(prompt)];
        Ok(self.complete(msgs).await?.content)
    }
}
