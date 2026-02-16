#![allow(dead_code)]

use serde::{Deserialize, Serialize};

// === List (GET /api/tags) ===

#[derive(Debug, Deserialize)]
pub struct TagsResponse {
    pub models: Vec<ModelInfo>,
}

#[derive(Debug, Deserialize)]
pub struct ModelInfo {
    pub name: String,
    pub size: u64,
    pub modified_at: String,
    pub digest: Option<String>,
    pub details: Option<ModelDetails>,
}

#[derive(Debug, Deserialize)]
pub struct ModelDetails {
    pub family: Option<String>,
    pub parameter_size: Option<String>,
    pub quantization_level: Option<String>,
}

// === PS (GET /api/ps) ===

#[derive(Debug, Deserialize)]
pub struct PsResponse {
    pub models: Vec<RunningModel>,
}

#[derive(Debug, Deserialize)]
pub struct RunningModel {
    pub name: String,
    pub size: u64,
    pub expires_at: Option<String>,
    pub details: Option<ModelDetails>,
}

// === Pull/Push streaming ===

#[derive(Debug, Deserialize)]
pub struct ProgressResponse {
    pub status: Option<String>,
    pub digest: Option<String>,
    pub total: Option<u64>,
    pub completed: Option<u64>,
    pub error: Option<String>,
}

// === Generate (POST /api/generate) ===

#[derive(Debug, Serialize, Default)]
pub struct GenerateRequest {
    pub model: String,
    pub prompt: String,
    pub stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub template: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "is_false")]
    pub raw: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keep_alive: Option<i64>,
}

fn is_false(v: &bool) -> bool {
    !*v
}

#[derive(Debug, Deserialize)]
pub struct GenerateResponse {
    pub response: Option<String>,
    pub done: Option<bool>,
    pub eval_count: Option<u64>,
    pub eval_duration: Option<u64>,
    pub total_duration: Option<u64>,
    pub prompt_eval_count: Option<u64>,
    pub prompt_eval_duration: Option<u64>,
    pub context: Option<serde_json::Value>,
    pub error: Option<String>,
}

// === Chat (POST /api/chat) ===

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Serialize, Default)]
pub struct ChatRequest {
    pub model: String,
    pub messages: Vec<ChatMessage>,
    pub stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct ChatResponse {
    pub message: Option<ChatMessage>,
    pub done: Option<bool>,
    pub eval_count: Option<u64>,
    pub eval_duration: Option<u64>,
    pub total_duration: Option<u64>,
    pub error: Option<String>,
}

// === Embed (POST /api/embed) ===

#[derive(Debug, Serialize)]
pub struct EmbedRequest {
    pub model: String,
    pub input: String,
    pub truncate: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct EmbedResponse {
    pub embeddings: Vec<Vec<f64>>,
}

// === Show (POST /api/show) ===

#[derive(Debug, Serialize)]
pub struct ShowRequest {
    pub name: String,
    #[serde(skip_serializing_if = "is_false")]
    pub verbose: bool,
}

#[derive(Debug, Deserialize)]
pub struct ShowResponse {
    pub modelfile: Option<String>,
    pub parameters: Option<String>,
    pub template: Option<String>,
    pub model_info: Option<serde_json::Value>,
}

// === Copy (POST /api/copy) ===

#[derive(Debug, Serialize)]
pub struct CopyRequest {
    pub source: String,
    pub destination: String,
}

// === Delete (DELETE /api/delete) ===

#[derive(Debug, Serialize)]
pub struct DeleteRequest {
    pub name: String,
}

// === Create (POST /api/create) ===

#[derive(Debug, Serialize)]
pub struct CreateRequest {
    pub name: String,
    pub stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modelfile: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quantize: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateResponse {
    pub status: Option<String>,
    pub error: Option<String>,
}

// === Push (POST /api/push) ===

#[derive(Debug, Serialize)]
pub struct PushRequest {
    pub name: String,
    pub stream: bool,
    #[serde(skip_serializing_if = "is_false")]
    pub insecure: bool,
}

// === Common error response ===

#[derive(Debug, Deserialize)]
pub struct ErrorResponse {
    pub error: String,
}
