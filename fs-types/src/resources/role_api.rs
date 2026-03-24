//! Typed request and response types for the standardized role APIs.
//!
//! Each role method (e.g. `user.create` for IAM, `page.get` for wiki) has
//! a typed request/response pair defined here.  These types define the
//! contract that every bridge for that role must satisfy.

use serde::{Deserialize, Serialize};

// ── IAM ───────────────────────────────────────────────────────────────────────

/// `user.create` — create a new user account.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IamUserCreate {
    pub username: String,
    pub email: String,
    pub display_name: Option<String>,
}

/// `user.get` / `user.list` / `user.update` — a user record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IamUser {
    pub id: String,
    pub username: String,
    pub email: String,
    pub display_name: Option<String>,
    pub active: bool,
}

/// `user.update` — fields to update (all optional).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IamUserUpdate {
    pub email: Option<String>,
    pub display_name: Option<String>,
    pub active: Option<bool>,
}

/// `group.create` — create a new group.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IamGroupCreate {
    pub name: String,
    pub description: Option<String>,
}

/// `group.list` / `group.get` — a group record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IamGroup {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub member_count: u32,
}

/// `group.add_member` — add a user to a group.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IamGroupAddMember {
    pub group_id: String,
    pub user_id: String,
}

// ── Wiki ──────────────────────────────────────────────────────────────────────

/// `page.create` — create a new wiki page.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WikiPageCreate {
    pub title: String,
    pub content: String,
    pub collection_id: Option<String>,
}

/// `page.get` — a full wiki page.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WikiPage {
    pub id: String,
    pub title: String,
    pub content: String,
    pub url: String,
    pub updated_at: String,
}

/// `page.list` — a brief page summary.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WikiPageSummary {
    pub id: String,
    pub title: String,
    pub url: String,
}

/// `page.search` — a search result entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WikiSearchResult {
    pub id: String,
    pub title: String,
    pub excerpt: String,
    pub url: String,
}

// ── Git ───────────────────────────────────────────────────────────────────────

/// `repo.create` — create a new repository.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitRepoCreate {
    pub name: String,
    pub description: Option<String>,
    pub private: bool,
}

/// `repo.get` / `repo.list` — a repository record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitRepo {
    pub id: String,
    pub name: String,
    pub full_name: String,
    pub clone_url: String,
    pub private: bool,
    pub default_branch: String,
}

/// `commit.list` — a single commit.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitCommit {
    pub sha: String,
    pub message: String,
    pub author: String,
    pub timestamp: String,
}

// ── Chat ──────────────────────────────────────────────────────────────────────

/// `message.send` — send a message to a channel.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessageSend {
    pub channel_id: String,
    pub text: String,
}

/// `channel.list` / `channel.get` — a channel record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatChannel {
    pub id: String,
    pub name: String,
    /// "text", "voice", or "dm".
    pub kind: String,
    pub unread: u32,
}

// ── Database ──────────────────────────────────────────────────────────────────

/// `query.execute` — execute a parameterized SQL query.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbQueryRequest {
    pub sql: String,
    pub params: Vec<serde_json::Value>,
}

/// `query.execute` response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbQueryResult {
    pub columns: Vec<String>,
    pub rows: Vec<Vec<serde_json::Value>>,
    pub rows_affected: u64,
}

/// `schema.list` — one table in the schema.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbSchemaTable {
    pub name: String,
    pub columns: Vec<String>,
}

// ── Cache ─────────────────────────────────────────────────────────────────────

/// `key.get` / `key.set` — a cache entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry {
    pub key: String,
    pub value: Option<String>,
    pub ttl_secs: Option<u64>,
}

/// `key.set` request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheSet {
    pub key: String,
    pub value: String,
    pub ttl_secs: Option<u64>,
}

// ── SMTP ──────────────────────────────────────────────────────────────────────

/// `mail.send` — send an email.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MailSend {
    pub from: String,
    pub to: Vec<String>,
    pub subject: String,
    pub body_text: Option<String>,
    pub body_html: Option<String>,
}

// ── LLM ───────────────────────────────────────────────────────────────────────

/// `completion.create` — generate a text completion.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmCompletionRequest {
    pub model: String,
    pub prompt: String,
    pub max_tokens: Option<u32>,
    pub temperature: Option<f32>,
}

/// `completion.create` response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmCompletionResponse {
    pub model: String,
    pub text: String,
    pub tokens_used: u32,
}

/// `model.list` — a single available model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmModel {
    pub id: String,
    pub name: String,
    pub context_length: u32,
}

// ── Map ───────────────────────────────────────────────────────────────────────

/// `tile.get` — request a map tile.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MapTileRequest {
    pub z: u32,
    pub x: u32,
    pub y: u32,
}

/// `search.geocode` — a geocoding result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeoLocation {
    pub lat: f64,
    pub lon: f64,
    pub display_name: String,
}

// ── Tasks ─────────────────────────────────────────────────────────────────────

/// `task.create` — create a new task.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskCreate {
    pub title: String,
    pub description: Option<String>,
    pub project_id: Option<String>,
    pub assignee_id: Option<String>,
    pub due_date: Option<String>,
}

/// `task.list` / `task.update` — a task record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub done: bool,
    pub assignee_id: Option<String>,
    pub due_date: Option<String>,
}

/// `task.update` — fields to update.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskUpdate {
    pub title: Option<String>,
    pub done: Option<bool>,
    pub assignee_id: Option<String>,
}

// ── Monitoring ────────────────────────────────────────────────────────────────

/// `metric.query` — query a time-series metric.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricQuery {
    pub metric: String,
    pub start: String,
    pub end: String,
    pub step: Option<String>,
}

/// A single metric data point.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricPoint {
    pub timestamp: String,
    pub value: f64,
}

// ── AlertSeverity ─────────────────────────────────────────────────────────────

/// Severity of an active monitoring alert.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AlertSeverity {
    Critical,
    Warning,
    Info,
}

impl AlertSeverity {
    /// Human-readable label.
    pub fn label(self) -> &'static str {
        match self {
            AlertSeverity::Critical => "critical",
            AlertSeverity::Warning => "warning",
            AlertSeverity::Info => "info",
        }
    }
}

// ── AlertState ────────────────────────────────────────────────────────────────

/// Lifecycle state of an active monitoring alert.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AlertState {
    Firing,
    Resolved,
}

impl AlertState {
    /// Human-readable label.
    pub fn label(self) -> &'static str {
        match self {
            AlertState::Firing => "firing",
            AlertState::Resolved => "resolved",
        }
    }

    /// `true` when the alert is currently active.
    pub fn is_active(self) -> bool {
        matches!(self, AlertState::Firing)
    }
}

/// `alert.list` — an active alert.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    pub id: String,
    pub name: String,
    pub severity: AlertSeverity,
    pub state: AlertState,
    pub message: String,
}

impl Alert {
    /// `true` when this alert is currently firing.
    pub fn is_active(&self) -> bool {
        self.state.is_active()
    }
}
