//! LLM provider abstraction for FreeSynergy.
//!
//! Provides a unified [`LlmProvider`] trait with implementations for
//! Ollama, Anthropic Claude, and any OpenAI-compatible API.
//! High-level task functions (`interpret_command`, `summarize_logs`, …) work
//! with any provider.
//!
//! # Features
//!
//! | feature        | adds                       | extra dep   |
//! |----------------|----------------------------|-------------|
//! | `ollama`       | [`OllamaProvider`]         | `reqwest`, `tokio` |
//! | `claude`       | [`ClaudeProvider`]         | `reqwest`, `tokio` |
//! | `openai-compat`| [`OpenAiCompatProvider`]   | `reqwest`, `tokio` |
//!
//! # Quick start
//!
//! ```rust,ignore
//! use fsn_llm::{OllamaProvider, tasks};
//!
//! let provider = OllamaProvider::new("http://localhost:11434", "llama3");
//! let summary = tasks::summarize_logs(&provider, &log_text).await?;
//! ```

pub mod error;
pub mod provider;
pub mod request;
pub mod tasks;

#[cfg(feature = "ollama")]
pub mod ollama;

#[cfg(feature = "claude")]
pub mod claude;

#[cfg(feature = "openai-compat")]
pub mod openai_compat;

// ── Flat re-exports ───────────────────────────────────────────────────────────

pub use error::LlmError;
pub use provider::LlmProvider;
pub use request::{LlmResponse, Message, Role, TokenUsage};
pub use tasks::{ErrorExplanation, InterpretedCommand};

#[cfg(feature = "ollama")]
pub use ollama::OllamaProvider;

#[cfg(feature = "claude")]
pub use claude::ClaudeProvider;

#[cfg(feature = "openai-compat")]
pub use openai_compat::OpenAiCompatProvider;
