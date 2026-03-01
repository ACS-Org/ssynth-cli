use anyhow::{Context, Result};
use reqwest::{Client, RequestBuilder, Response, StatusCode};

use crate::config::AuthToken;
use crate::error::CliError;
use crate::models::ApiErrorResponse;

pub struct ApiClient {
    http: Client,
    base_url: String,
    auth: Option<AuthToken>,
}

impl ApiClient {
    pub fn new(base_url: &str, auth: Option<AuthToken>) -> Result<Self> {
        let http = Client::builder()
            .user_agent(format!("ssynth-cli/{}", env!("CARGO_PKG_VERSION")))
            .build()
            .context("Failed to create HTTP client")?;
        Ok(Self {
            http,
            base_url: base_url.trim_end_matches('/').to_string(),
            auth,
        })
    }

    pub fn url(&self, path: &str) -> String {
        format!("{}{path}", self.base_url)
    }

    fn apply_auth(&self, req: RequestBuilder) -> RequestBuilder {
        match &self.auth {
            Some(AuthToken::ApiKey(key)) => req.header("x-api-key", key),
            Some(AuthToken::Jwt(token)) => req.bearer_auth(token),
            None => req,
        }
    }

    pub fn get(&self, path: &str) -> RequestBuilder {
        let req = self.http.get(self.url(path));
        self.apply_auth(req)
    }

    pub fn post(&self, path: &str) -> RequestBuilder {
        let req = self.http.post(self.url(path));
        self.apply_auth(req)
    }

    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    pub fn auth_token(&self) -> Option<&AuthToken> {
        self.auth.as_ref()
    }
}

/// Check response status and parse API error body if not successful.
pub async fn check_response(resp: Response) -> Result<Response, CliError> {
    let status = resp.status();
    if status.is_success() {
        return Ok(resp);
    }

    let status_code = status.as_u16();
    let body = resp.text().await.unwrap_or_default();

    if let Ok(api_err) = serde_json::from_str::<ApiErrorResponse>(&body) {
        return Err(CliError::Api {
            status: status_code,
            error_code: api_err.error,
            message: api_err.message,
        });
    }

    let message = if body.is_empty() {
        status
            .canonical_reason()
            .unwrap_or("Unknown error")
            .to_string()
    } else {
        body
    };

    Err(CliError::Api {
        status: status_code,
        error_code: match status {
            StatusCode::UNAUTHORIZED => "unauthorized".to_string(),
            StatusCode::FORBIDDEN => "forbidden".to_string(),
            StatusCode::NOT_FOUND => "not_found".to_string(),
            StatusCode::TOO_MANY_REQUESTS => "rate_limited".to_string(),
            _ => "error".to_string(),
        },
        message,
    })
}
