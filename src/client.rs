use anyhow::{Context, Result};
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};
use reqwest::StatusCode;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::time::{Duration, Instant};

use crate::api::openai;
use crate::api::types::{
    error_message, ChatMessage, ChatRequest, ChatResponse, EmbedRequest, EmbedResponse,
    GenerateRequest, GenerateResponse, ModelInfo, TagsResponse,
};
use crate::config::{BackendKind, Endpoint};
use crate::stream::{ChatStream, GenerateStream, JsonStream, OpenAiChatStream, SseStream};

/// HTTP client for a configured backend. Speaks either the native Ollama API or
/// an OpenAI-compatible API (e.g. LiteLLM); operations both support (list
/// models, chat, generate, embed) go through the backend-agnostic methods.
#[derive(Clone)]
pub struct OllamaClient {
    client: reqwest::Client,
    base_url: String,
    kind: BackendKind,
}

impl OllamaClient {
    pub fn new(endpoint: &Endpoint) -> Result<Self> {
        let mut headers = HeaderMap::new();
        if let Some(key) = &endpoint.api_key {
            let mut value = HeaderValue::from_str(&format!("Bearer {}", key))
                .context("API key contains characters not allowed in an HTTP header")?;
            value.set_sensitive(true);
            headers.insert(AUTHORIZATION, value);
        }

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(120))
            .default_headers(headers)
            .build()
            .context("Failed to create HTTP client")?;

        let mut base_url = endpoint.url.trim_end_matches('/');
        if endpoint.kind == BackendKind::OpenAi {
            // Accept the URL with or without the /v1 suffix
            base_url = base_url.strip_suffix("/v1").unwrap_or(base_url);
        }

        Ok(Self {
            client,
            base_url: base_url.to_string(),
            kind: endpoint.kind,
        })
    }

    /// Test connectivity by listing models with a short timeout
    pub async fn test_connection(&self) -> Result<()> {
        let (path, what) = match self.kind {
            BackendKind::Ollama => ("/api/tags", "Ollama instance"),
            BackendKind::OpenAi => ("/v1/models", "OpenAI-compatible endpoint"),
        };
        let url = format!("{}{}", self.base_url, path);
        let response = self
            .client
            .get(&url)
            .timeout(Duration::from_secs(5))
            .send()
            .await
            .with_context(|| {
                format!(
                    "Cannot connect to {} at {}\nPlease check the URL and ensure the instance is running.",
                    what, self.base_url
                )
            })?;
        if matches!(
            response.status(),
            StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN
        ) {
            anyhow::bail!(
                "Authentication failed for {} (HTTP {})\nCheck the profile's API key.",
                self.base_url,
                response.status()
            );
        }
        Ok(())
    }

    pub fn url(&self) -> &str {
        &self.base_url
    }

    /// Whether Ollama-only features (pull, ps, show, ...) are available
    pub fn supports_ollama_api(&self) -> bool {
        self.kind == BackendKind::Ollama
    }

    /// Fail with a clear message if `feature` needs the native Ollama API
    pub fn require_ollama(&self, feature: &str) -> Result<()> {
        if !self.supports_ollama_api() {
            anyhow::bail!(
                "{} is not supported by OpenAI-compatible backends ({})",
                feature,
                self.base_url
            );
        }
        Ok(())
    }

    // === Backend-agnostic operations ===

    pub async fn list_models(&self) -> Result<Vec<ModelInfo>> {
        match self.kind {
            BackendKind::Ollama => Ok(self.get::<TagsResponse>("/api/tags").await?.models),
            BackendKind::OpenAi => {
                let list: openai::ModelList = self.get("/v1/models").await?;
                Ok(list.data.into_iter().map(ModelInfo::from).collect())
            }
        }
    }

    /// Non-streaming chat
    pub async fn chat(&self, request: &ChatRequest) -> Result<ChatResponse> {
        match self.kind {
            BackendKind::Ollama => self.post("/api/chat", request).await,
            BackendKind::OpenAi => {
                let started = Instant::now();
                let body = openai::ChatCompletionRequest::from_ollama(request, false)?;
                let completion: openai::ChatCompletion =
                    self.post("/v1/chat/completions", &body).await?;
                // No server-side timing; wall time includes prompt processing
                let elapsed = started.elapsed().as_nanos() as u64;
                Ok(ChatResponse {
                    message: Some(ChatMessage {
                        role: "assistant".to_string(),
                        content: completion
                            .choices
                            .into_iter()
                            .next()
                            .and_then(|c| c.message.content)
                            .unwrap_or_default(),
                    }),
                    done: Some(true),
                    eval_count: completion.usage.and_then(|u| u.completion_tokens),
                    eval_duration: Some(elapsed),
                    total_duration: Some(elapsed),
                    error: None,
                })
            }
        }
    }

    pub async fn chat_stream(&self, request: &ChatRequest) -> Result<ChatStream> {
        match self.kind {
            BackendKind::Ollama => Ok(ChatStream::Ollama(
                self.post_stream("/api/chat", request).await?,
            )),
            BackendKind::OpenAi => {
                let started = Instant::now();
                let body = openai::ChatCompletionRequest::from_ollama(request, true)?;
                let response = self.post_streaming("/v1/chat/completions", &body).await?;
                Ok(ChatStream::OpenAi(OpenAiChatStream::new(
                    SseStream::from_response(response),
                    started,
                )))
            }
        }
    }

    /// Non-streaming generate
    pub async fn generate(&self, request: &GenerateRequest) -> Result<GenerateResponse> {
        match self.kind {
            BackendKind::Ollama => self.post("/api/generate", request).await,
            BackendKind::OpenAi => Ok(self.chat(&generate_as_chat(request)?).await?.into()),
        }
    }

    pub async fn generate_stream(&self, request: &GenerateRequest) -> Result<GenerateStream> {
        match self.kind {
            BackendKind::Ollama => Ok(GenerateStream::Ollama(
                self.post_stream("/api/generate", request).await?,
            )),
            BackendKind::OpenAi => Ok(GenerateStream::Chat(
                self.chat_stream(&generate_as_chat(request)?).await?,
            )),
        }
    }

    pub async fn embed(&self, request: &EmbedRequest) -> Result<EmbedResponse> {
        match self.kind {
            BackendKind::Ollama => self.post("/api/embed", request).await,
            BackendKind::OpenAi => {
                let body = openai::EmbeddingRequest {
                    model: &request.model,
                    input: &request.input,
                    params: openai::params_from_options(request.options.as_ref())?,
                };
                let response: openai::EmbeddingResponse =
                    self.post("/v1/embeddings", &body).await?;
                Ok(EmbedResponse {
                    embeddings: response.data.into_iter().map(|e| e.embedding).collect(),
                })
            }
        }
    }

    // === Raw HTTP helpers ===

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
        let response = self.post_streaming(path, body).await?;
        Ok(JsonStream::from_response(response))
    }

    /// POST with the long streaming timeout, failing on a non-success status
    async fn post_streaming<R: Serialize>(
        &self,
        path: &str,
        body: &R,
    ) -> Result<reqwest::Response> {
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
            anyhow::bail!("HTTP {}: {}", status, error_message(&body_text));
        }

        Ok(response)
    }
}

/// Express an Ollama generate request as a chat request (system + user message)
fn generate_as_chat(request: &GenerateRequest) -> Result<ChatRequest> {
    if request.template.is_some() || request.context.is_some() || request.raw {
        anyhow::bail!(
            "--template, --context and --raw are not supported by OpenAI-compatible backends"
        );
    }

    let mut messages = Vec::new();
    if let Some(system) = &request.system {
        messages.push(ChatMessage {
            role: "system".to_string(),
            content: system.clone(),
        });
    }
    messages.push(ChatMessage {
        role: "user".to_string(),
        content: request.prompt.clone(),
    });

    Ok(ChatRequest {
        model: request.model.clone(),
        messages,
        stream: request.stream,
        format: request.format.clone(),
        options: request.options.clone(),
    })
}

/// Check HTTP response status and extract error message if not successful
async fn check_response(response: reqwest::Response) -> Result<reqwest::Response> {
    if response.status().is_success() {
        return Ok(response);
    }
    let status = response.status();
    let body = response.text().await.unwrap_or_default();
    let msg = error_message(&body);

    match status {
        StatusCode::NOT_FOUND => anyhow::bail!("Not found: {}", msg),
        StatusCode::BAD_REQUEST => anyhow::bail!("Bad request: {}", msg),
        StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN => {
            anyhow::bail!("Unauthorized (HTTP {}): {}\nCheck the profile's API key.", status, msg)
        }
        _ => anyhow::bail!("HTTP {}: {}", status, msg),
    }
}
