// HTTP bridge base type for bridge implementors.

use std::time::Duration;

use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};
use serde::{de::DeserializeOwned, Serialize};

use fs_error::FsError;

use crate::bridge::BridgeConfig;

// ── HttpBridge ────────────────────────────────────────────────────────────────

/// Base HTTP client for bridge implementations.
///
/// Bridge implementors should embed this via composition rather than re-implementing
/// HTTP mechanics themselves.
pub struct HttpBridge {
    config: BridgeConfig,
    client: reqwest::Client,
}

impl HttpBridge {
    /// Build from config. Constructs a `reqwest::Client` with the configured timeout
    /// and, if present, a default `Authorization: Bearer <token>` header.
    pub fn new(config: BridgeConfig) -> Result<Self, FsError> {
        let mut headers = HeaderMap::new();

        if let Some(token) = &config.token {
            let value = HeaderValue::from_str(&format!("Bearer {token}"))
                .map_err(|e| FsError::config(format!("invalid token for bearer header: {e}")))?;
            headers.insert(AUTHORIZATION, value);
        }

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(config.timeout_secs))
            .default_headers(headers)
            .build()
            .map_err(|e| FsError::network(format!("failed to build HTTP client: {e}")))?;

        Ok(Self { config, client })
    }

    /// GET `{base_url}/{path}` and deserialize the JSON response body.
    pub async fn get<T: DeserializeOwned>(&self, path: &str) -> Result<T, FsError> {
        let url = self.url(path);
        let resp = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| FsError::network(format!("GET {url}: {e}")))?;
        self.json(resp, &url).await
    }

    /// POST `{base_url}/{path}` with a JSON body and deserialize the JSON response.
    pub async fn post<B: Serialize, T: DeserializeOwned>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T, FsError> {
        let url = self.url(path);
        let resp = self
            .client
            .post(&url)
            .json(body)
            .send()
            .await
            .map_err(|e| FsError::network(format!("POST {url}: {e}")))?;
        self.json(resp, &url).await
    }

    /// PUT `{base_url}/{path}` with a JSON body and deserialize the JSON response.
    pub async fn put<B: Serialize, T: DeserializeOwned>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T, FsError> {
        let url = self.url(path);
        let resp = self
            .client
            .put(&url)
            .json(body)
            .send()
            .await
            .map_err(|e| FsError::network(format!("PUT {url}: {e}")))?;
        self.json(resp, &url).await
    }

    /// DELETE `{base_url}/{path}`. Returns an error if the response is not successful.
    pub async fn delete(&self, path: &str) -> Result<(), FsError> {
        let url = self.url(path);
        let resp = self
            .client
            .delete(&url)
            .send()
            .await
            .map_err(|e| FsError::network(format!("DELETE {url}: {e}")))?;
        if !resp.status().is_success() {
            let status = resp.status();
            return Err(FsError::network(format!("DELETE {url}: HTTP {status}")));
        }
        Ok(())
    }

    /// Access the underlying `reqwest::Client` for custom requests.
    pub fn client(&self) -> &reqwest::Client {
        &self.client
    }

    /// Access the bridge configuration.
    pub fn config(&self) -> &BridgeConfig {
        &self.config
    }

    // ── helpers ───────────────────────────────────────────────────────────────

    fn url(&self, path: &str) -> String {
        let base = self.config.base_url.trim_end_matches('/');
        let path = path.trim_start_matches('/');
        format!("{base}/{path}")
    }

    async fn json<T: DeserializeOwned>(
        &self,
        resp: reqwest::Response,
        url: &str,
    ) -> Result<T, FsError> {
        let status = resp.status();
        if !status.is_success() {
            return Err(FsError::network(format!("{url}: HTTP {status}")));
        }
        resp.json::<T>()
            .await
            .map_err(|e| FsError::parse(format!("JSON decode from {url}: {e}")))
    }
}
