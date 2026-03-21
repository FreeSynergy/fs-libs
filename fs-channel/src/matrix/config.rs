// fs-channel/src/matrix/config.rs — Matrix adapter configuration.

use serde::{Deserialize, Serialize};

/// Configuration for the Matrix adapter.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatrixConfig {
    /// Homeserver URL, e.g. `"https://matrix.org"`.
    pub homeserver_url: String,
    /// Full Matrix user ID, e.g. `"@bot:matrix.org"`.
    pub user_id: String,
    /// Password for initial login. Prefer `access_token` for production.
    #[serde(default, skip_serializing)]
    pub password: Option<String>,
    /// Pre-obtained access token — takes precedence over `password`.
    #[serde(default, skip_serializing)]
    pub access_token: Option<String>,
    /// Path for the persistent session store (SQLite).
    /// Defaults to `"./fs-matrix-session"`.
    #[serde(default = "MatrixConfig::default_store_path")]
    pub store_path: String,
}

impl MatrixConfig {
    fn default_store_path() -> String {
        "./fs-matrix-session".to_string()
    }
}
