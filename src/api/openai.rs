//! Wire types for OpenAI-compatible backends (e.g. LiteLLM), plus translation
//! from the Ollama request types the rest of the app uses.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};

use super::types::{ChatMessage, ChatRequest, ModelInfo};

// === Models (GET /v1/models) ===

#[derive(Debug, Deserialize)]
pub struct ModelList {
    pub data: Vec<Model>,
}

#[derive(Debug, Deserialize)]
pub struct Model {
    pub id: String,
    pub created: Option<i64>,
}

impl From<Model> for ModelInfo {
    fn from(model: Model) -> Self {
        ModelInfo {
            name: model.id,
            size: None,
            modified_at: model
                .created
                .filter(|&ts| ts > 0)
                .and_then(|ts| chrono::DateTime::from_timestamp(ts, 0))
                .map(|dt| dt.to_rfc3339()),
            digest: None,
            details: None,
        }
    }
}

// === Chat (POST /v1/chat/completions) ===

#[derive(Debug, Serialize)]
pub struct ChatCompletionRequest<'a> {
    pub model: &'a str,
    pub messages: &'a [ChatMessage],
    pub stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream_options: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_format: Option<Value>,
    /// Sampling parameters translated from Ollama `options`
    #[serde(flatten)]
    pub params: Map<String, Value>,
}

impl<'a> ChatCompletionRequest<'a> {
    pub fn from_ollama(request: &'a ChatRequest, stream: bool) -> Result<Self> {
        Ok(ChatCompletionRequest {
            model: &request.model,
            messages: &request.messages,
            stream,
            // Ask for a final usage chunk so tokens/sec can be reported
            stream_options: stream.then(|| json!({ "include_usage": true })),
            response_format: response_format(request.format.as_deref())?,
            params: params_from_options(request.options.as_ref())?,
        })
    }
}

#[derive(Debug, Deserialize)]
pub struct ChatCompletion {
    pub choices: Vec<CompletionChoice>,
    pub usage: Option<Usage>,
}

#[derive(Debug, Deserialize)]
pub struct CompletionChoice {
    pub message: CompletionMessage,
}

#[derive(Debug, Deserialize, Default)]
pub struct CompletionMessage {
    pub content: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ChatCompletionChunk {
    #[serde(default)]
    pub choices: Vec<ChunkChoice>,
    pub usage: Option<Usage>,
    pub error: Option<Value>,
}

#[derive(Debug, Deserialize)]
pub struct ChunkChoice {
    #[serde(default)]
    pub delta: CompletionMessage,
}

#[derive(Debug, Deserialize)]
pub struct Usage {
    pub completion_tokens: Option<u64>,
}

// === Embeddings (POST /v1/embeddings) ===

#[derive(Debug, Serialize)]
pub struct EmbeddingRequest<'a> {
    pub model: &'a str,
    pub input: &'a str,
    #[serde(flatten)]
    pub params: Map<String, Value>,
}

#[derive(Debug, Deserialize)]
pub struct EmbeddingResponse {
    pub data: Vec<Embedding>,
}

#[derive(Debug, Deserialize)]
pub struct Embedding {
    pub embedding: Vec<f64>,
}

// === Translation helpers ===

/// Map Ollama `options` onto top-level OpenAI params. Keys with a different
/// name are renamed; everything else is passed through unchanged.
pub fn params_from_options(options: Option<&Value>) -> Result<Map<String, Value>> {
    let Some(options) = options else {
        return Ok(Map::new());
    };
    let options = options
        .as_object()
        .context("--options must be a JSON object")?;
    Ok(options
        .iter()
        .map(|(key, value)| {
            let key = match key.as_str() {
                "num_predict" => "max_tokens",
                other => other,
            };
            (key.to_string(), value.clone())
        })
        .collect())
}

/// Map Ollama `format` ("json" or a JSON schema) onto OpenAI `response_format`
fn response_format(format: Option<&str>) -> Result<Option<Value>> {
    match format {
        None => Ok(None),
        Some("json") => Ok(Some(json!({ "type": "json_object" }))),
        Some(schema) => {
            let schema: Value = serde_json::from_str(schema)
                .context("--format must be \"json\" or a JSON schema")?;
            Ok(Some(json!({
                "type": "json_schema",
                "json_schema": { "name": "response", "schema": schema },
            })))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chat_request_translates_options_and_format() {
        let request = ChatRequest {
            model: "gpt-4o".to_string(),
            messages: vec![ChatMessage {
                role: "user".to_string(),
                content: "hi".to_string(),
            }],
            stream: true,
            format: Some("json".to_string()),
            options: Some(json!({ "temperature": 0.2, "num_predict": 50 })),
        };

        let body = serde_json::to_value(ChatCompletionRequest::from_ollama(&request, true).unwrap())
            .unwrap();

        assert_eq!(body["model"], "gpt-4o");
        assert_eq!(body["stream"], true);
        assert_eq!(body["stream_options"]["include_usage"], true);
        assert_eq!(body["response_format"]["type"], "json_object");
        assert_eq!(body["temperature"], 0.2);
        assert_eq!(body["max_tokens"], 50);
        assert!(body.get("num_predict").is_none());
    }

    #[test]
    fn non_streaming_request_omits_stream_options() {
        let request = ChatRequest {
            model: "m".to_string(),
            ..Default::default()
        };
        let body = serde_json::to_value(ChatCompletionRequest::from_ollama(&request, false).unwrap())
            .unwrap();
        assert!(body.get("stream_options").is_none());
        assert!(body.get("response_format").is_none());
    }

    #[test]
    fn chunk_parses_delta_usage_and_error() {
        let chunk: ChatCompletionChunk = serde_json::from_str(
            r#"{"choices":[{"index":0,"delta":{"content":"Hel"}}]}"#,
        )
        .unwrap();
        assert_eq!(chunk.choices[0].delta.content.as_deref(), Some("Hel"));

        let usage: ChatCompletionChunk =
            serde_json::from_str(r#"{"choices":[],"usage":{"completion_tokens":7}}"#).unwrap();
        assert_eq!(usage.usage.unwrap().completion_tokens, Some(7));

        let role_only: ChatCompletionChunk =
            serde_json::from_str(r#"{"choices":[{"delta":{"role":"assistant"}}]}"#).unwrap();
        assert!(role_only.choices[0].delta.content.is_none());

        let err: ChatCompletionChunk =
            serde_json::from_str(r#"{"error":{"message":"bad key"}}"#).unwrap();
        assert_eq!(super::super::types::error_text(&err.error.unwrap()), "bad key");
    }
}
