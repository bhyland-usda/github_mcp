use reqwest::{Client, Method};
use serde_json::{Value, json};
use std::time::Duration;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum GithubError {
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),
    #[error("JSON parse failed: {0}")]
    Json(#[from] serde_json::Error),
    #[error("GitHub API error {status}: {body}")]
    Api { status: u16, body: String },
}

impl From<GithubError> for rmcp::ErrorData {
    fn from(err: GithubError) -> Self {
        match err {
            GithubError::Api { status, body } => rmcp::ErrorData::internal_error(
                format!("GitHub API returned HTTP {status}"),
                Some(json!({ "details": body })),
            ),
            other => rmcp::ErrorData::internal_error(other.to_string(), None),
        }
    }
}

#[derive(Clone)]
pub struct GithubClient {
    http: Client,
    token: Option<String>,
}

impl GithubClient {
    pub fn new(token: Option<String>) -> Self {
        let http = Client::builder()
            .timeout(Duration::from_secs(30))
            .connect_timeout(Duration::from_secs(10))
            .build()
            .expect("reqwest client should build");

        Self { http, token }
    }

    pub async fn get(&self, path: &str) -> Result<Value, GithubError> {
        self.request(Method::GET, path, None).await
    }

    pub async fn post(&self, path: &str, body: Value) -> Result<Value, GithubError> {
        self.request(Method::POST, path, Some(body)).await
    }

    async fn request(
        &self,
        method: Method,
        path: &str,
        body: Option<Value>,
    ) -> Result<Value, GithubError> {
        let url = format!("https://api.github.com{path}");

        let mut req = self
            .http
            .request(method, &url)
            .header("Accept", "application/vnd.github+json")
            .header("X-GitHub-Api-Version", "2022-11-28")
            .header(
                "User-Agent",
                concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION")),
            );

        if let Some(token) = &self.token {
            req = req.bearer_auth(token);
        }

        if let Some(body) = body {
            req = req.json(&body);
        }

        let response = req.send().await?;
        let status = response.status();
        let text = response.text().await?;

        if !status.is_success() {
            return Err(GithubError::Api {
                status: status.as_u16(),
                body: text,
            });
        }

        Ok(serde_json::from_str(&text)?)
    }
}
