use anyhow::Result;
use futures::TryStreamExt;
use serde::de::DeserializeOwned;
use std::pin::Pin;
use std::time::Instant;
use tokio::io::AsyncBufReadExt;

use crate::api::openai::ChatCompletionChunk;
use crate::api::types::{error_text, ChatMessage, ChatResponse, GenerateResponse};

type LineReader = tokio::io::Lines<
    tokio::io::BufReader<
        tokio_util::io::StreamReader<
            Pin<Box<dyn futures::Stream<Item = std::io::Result<bytes::Bytes>> + Send>>,
            bytes::Bytes,
        >,
    >,
>;

fn line_reader(response: reqwest::Response) -> LineReader {
    let byte_stream = response
        .bytes_stream()
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e));

    let pinned: Pin<Box<dyn futures::Stream<Item = std::io::Result<bytes::Bytes>> + Send>> =
        Box::pin(byte_stream);

    let stream_reader = tokio_util::io::StreamReader::new(pinned);
    tokio::io::BufReader::new(stream_reader).lines()
}

/// Reads newline-delimited JSON from a streaming HTTP response.
pub struct JsonStream {
    lines: LineReader,
}

impl JsonStream {
    pub fn from_response(response: reqwest::Response) -> Self {
        Self {
            lines: line_reader(response),
        }
    }

    /// Read and deserialize the next JSON line. Returns None on EOF.
    pub async fn next_json<T: DeserializeOwned>(&mut self) -> Result<Option<T>> {
        loop {
            match self.lines.next_line().await? {
                None => return Ok(None),
                Some(ref line) if line.trim().is_empty() => continue,
                Some(line) => {
                    let parsed = serde_json::from_str(&line)?;
                    return Ok(Some(parsed));
                }
            }
        }
    }
}

/// Reads `data:` payloads from a server-sent events response.
pub struct SseStream {
    lines: LineReader,
}

impl SseStream {
    pub fn from_response(response: reqwest::Response) -> Self {
        Self {
            lines: line_reader(response),
        }
    }

    /// Next `data:` payload. Returns None on `[DONE]` or EOF.
    pub async fn next_data(&mut self) -> Result<Option<String>> {
        while let Some(line) = self.lines.next_line().await? {
            let Some(data) = line.strip_prefix("data:") else {
                continue;
            };
            let data = data.trim();
            if data == "[DONE]" {
                return Ok(None);
            }
            if !data.is_empty() {
                return Ok(Some(data.to_string()));
            }
        }
        Ok(None)
    }
}

/// Streaming chat response, normalized to Ollama-style chunks for either backend.
/// The final chunk has `done == Some(true)`.
pub enum ChatStream {
    Ollama(JsonStream),
    OpenAi(OpenAiChatStream),
}

impl ChatStream {
    pub async fn next_chunk(&mut self) -> Result<Option<ChatResponse>> {
        match self {
            ChatStream::Ollama(stream) => stream.next_json().await,
            ChatStream::OpenAi(stream) => stream.next_chunk().await,
        }
    }
}

pub struct OpenAiChatStream {
    sse: SseStream,
    started: Instant,
    last_token: Option<Instant>,
    completion_tokens: Option<u64>,
    finished: bool,
}

impl OpenAiChatStream {
    pub fn new(sse: SseStream, started: Instant) -> Self {
        Self {
            sse,
            started,
            last_token: None,
            completion_tokens: None,
            finished: false,
        }
    }

    async fn next_chunk(&mut self) -> Result<Option<ChatResponse>> {
        while !self.finished {
            let Some(data) = self.sse.next_data().await? else {
                self.finished = true;
                return Ok(Some(self.done_chunk()));
            };

            let chunk: ChatCompletionChunk = serde_json::from_str(&data)?;
            if let Some(err) = &chunk.error {
                self.finished = true;
                return Ok(Some(ChatResponse {
                    error: Some(error_text(err)),
                    ..Default::default()
                }));
            }
            if let Some(tokens) = chunk.usage.and_then(|u| u.completion_tokens) {
                self.completion_tokens = Some(tokens);
            }

            let content: String = chunk
                .choices
                .iter()
                .flatten()
                .filter_map(|c| c.delta.as_ref()?.content.as_deref())
                .collect();
            if content.is_empty() {
                continue;
            }

            self.last_token = Some(Instant::now());
            return Ok(Some(ChatResponse {
                message: Some(ChatMessage {
                    role: "assistant".to_string(),
                    content,
                }),
                done: Some(false),
                ..Default::default()
            }));
        }
        Ok(None)
    }

    /// Final chunk with timing measured client-side, since OpenAI streams carry
    /// token counts (when the backend honors `include_usage`) but no durations.
    /// Generation time runs from the request to the last token: chunk arrival
    /// times are unreliable when a proxy buffers the stream, and this includes
    /// prompt processing, so the rate reads slightly low rather than absurdly high.
    fn done_chunk(&self) -> ChatResponse {
        let eval_duration = self
            .last_token
            .map(|last| (last - self.started).as_nanos() as u64)
            .filter(|&nanos| nanos > 0);
        ChatResponse {
            done: Some(true),
            eval_count: self.completion_tokens,
            eval_duration,
            total_duration: Some(self.started.elapsed().as_nanos() as u64),
            ..Default::default()
        }
    }
}

/// Streaming generate response. OpenAI backends have no completion endpoint
/// equivalent to /api/generate, so generation runs through chat.
pub enum GenerateStream {
    Ollama(JsonStream),
    Chat(ChatStream),
}

impl GenerateStream {
    pub async fn next_chunk(&mut self) -> Result<Option<GenerateResponse>> {
        match self {
            GenerateStream::Ollama(stream) => stream.next_json().await,
            GenerateStream::Chat(stream) => Ok(stream.next_chunk().await?.map(Into::into)),
        }
    }
}
