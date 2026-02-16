use anyhow::Result;
use futures::TryStreamExt;
use serde::de::DeserializeOwned;
use std::pin::Pin;
use tokio::io::AsyncBufReadExt;

/// Reads newline-delimited JSON from a streaming HTTP response.
pub struct JsonStream {
    lines: tokio::io::Lines<
        tokio::io::BufReader<
            tokio_util::io::StreamReader<
                Pin<Box<dyn futures::Stream<Item = std::io::Result<bytes::Bytes>> + Send>>,
                bytes::Bytes,
            >,
        >,
    >,
}

impl JsonStream {
    pub fn from_response(response: reqwest::Response) -> Self {
        let byte_stream = response
            .bytes_stream()
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e));

        let pinned: Pin<
            Box<dyn futures::Stream<Item = std::io::Result<bytes::Bytes>> + Send>,
        > = Box::pin(byte_stream);

        let stream_reader = tokio_util::io::StreamReader::new(pinned);
        let buf_reader = tokio::io::BufReader::new(stream_reader);
        let lines = buf_reader.lines();
        Self { lines }
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
