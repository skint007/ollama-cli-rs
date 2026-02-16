use anyhow::{Context, Result};
use reqwest::StatusCode;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::time::Duration;

use crate::api::types::ErrorResponse;
use crate::stream::JsonStream;

#[derive(Clone)]
pub struct OllamaClient {
    client: reqwest::Client,
    base_url: String,
}

impl OllamaClient {
    pub fn new(base_url: &str) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(120))
            .build()
            .expect("Failed to create HTTP client");
        Self {
            client,
            base_url: base_url.trim_end_matches('/').to_string(),
        }
    }

    /// Test connectivity by hitting /api/tags with a short timeout
    pub async fn test_connection(&self) -> Result<()> {
        let url = format!("{}/api/tags", self.base_url);
        self.client
            .get(&url)
            .timeout(Duration::from_secs(5))
            .send()
            .await
            .with_context(|| {
                format!(
                    "Cannot connect to Ollama instance at {}\nPlease check the URL and ensure the instance is running.",
                    self.base_url
                )
            })?;
        Ok(())
    }

    pub fn url(&self) -> &str {
        &self.base_url
    }

    /// GET request, deserialize JSON response
    pub async fn get<T: DeserializeOwned>(&self, path: &str) -> Result<T> {
        let url = format!("{}{}", self.base_url, path);
        let response = self.client.get(&url).send().await?;
        let response = check_response(response).await?;
        let body = response.json::<T>().await?;
        Ok(body)
    }

    /// POST request, deserialize JSON response
    pub async fn post<R: Serialize, T: DeserializeOwned>(
        &self,
        path: &str,
        body: &R,
    ) -> Result<T> {
        let url = format!("{}{}", self.base_url, path);
        let response = self.client.post(&url).json(body).send().await?;
        let response = check_response(response).await?;
        let result = response.json::<T>().await?;
        Ok(result)
    }

    /// POST request, return raw response for status code checking
    pub async fn post_raw<R: Serialize>(
        &self,
        path: &str,
        body: &R,
    ) -> Result<reqwest::Response> {
        let url = format!("{}{}", self.base_url, path);
        let response = self.client.post(&url).json(body).send().await?;
        Ok(response)
    }

    /// DELETE request with JSON body
    pub async fn delete<R: Serialize>(
        &self,
        path: &str,
        body: &R,
    ) -> Result<reqwest::Response> {
        let url = format!("{}{}", self.base_url, path);
        let response = self.client.delete(&url).json(body).send().await?;
        Ok(response)
    }

    /// POST request returning a streaming JSON reader
    pub async fn post_stream<R: Serialize>(
        &self,
        path: &str,
        body: &R,
    ) -> Result<JsonStream> {
        let url = format!("{}{}", self.base_url, path);
        let response = self
            .client
            .post(&url)
            .json(body)
            .timeout(Duration::from_secs(600))
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body_text = response.text().await.unwrap_or_default();
            let msg = serde_json::from_str::<ErrorResponse>(&body_text)
                .map(|e| e.error)
                .unwrap_or(body_text);
            anyhow::bail!("HTTP {}: {}", status, msg);
        }

        Ok(JsonStream::from_response(response))
    }
}

/// Check HTTP response status and extract error message if not successful
async fn check_response(response: reqwest::Response) -> Result<reqwest::Response> {
    if response.status().is_success() {
        return Ok(response);
    }
    let status = response.status();
    let body = response.text().await.unwrap_or_default();
    let msg = serde_json::from_str::<ErrorResponse>(&body)
        .map(|e| e.error)
        .unwrap_or(body);

    match status {
        StatusCode::NOT_FOUND => anyhow::bail!("Not found: {}", msg),
        StatusCode::BAD_REQUEST => anyhow::bail!("Bad request: {}", msg),
        _ => anyhow::bail!("HTTP {}: {}", status, msg),
    }
}
