use anyhow::Result;
use crossterm::event::{EventStream, Event as CrosstermEvent};
use futures::StreamExt;
use std::time::Duration;
use tokio::sync::mpsc;

use crate::api::types::{ModelInfo, RunningModel, ShowResponse};
use crate::tui::app::{BenchmarkResult, LibraryModel};

#[derive(Debug)]
pub enum Event {
    Input(CrosstermEvent),
    Tick,
    Api(ApiEvent),
}

#[derive(Debug)]
pub enum ApiEvent {
    ChatToken(String),
    ChatDone {
        eval_count: Option<u64>,
        eval_duration: Option<u64>,
        total_duration: Option<u64>,
    },
    ChatError(String),
    ModelsLoaded(Vec<ModelInfo>),
    RunningModelsLoaded(Vec<RunningModel>),
    ModelDetailLoaded {
        name: String,
        response: ShowResponse,
    },
    ModelDeleted(String),
    ModelUnloaded(String),
    ModelCopied {
        source: String,
        destination: String,
    },
    PullProgress {
        status: String,
        digest: Option<String>,
        total: Option<u64>,
        completed: Option<u64>,
    },
    PullComplete(String),
    PullError(String),
    LibraryLoaded(Vec<LibraryModel>),
    LibraryError(String),
    BenchmarkProgress {
        model: String,
        round: u32,
        total_rounds: u32,
    },
    BenchmarkModelDone(BenchmarkResult),
    BenchmarkComplete,
    BenchmarkError(String),
    ConnectionStatus(bool),
    Error(String),
}

pub struct EventHandler {
    crossterm_events: EventStream,
    tick_interval: tokio::time::Interval,
    api_rx: mpsc::UnboundedReceiver<ApiEvent>,
}

impl EventHandler {
    pub fn new(api_rx: mpsc::UnboundedReceiver<ApiEvent>, tick_rate_ms: u64) -> Self {
        Self {
            crossterm_events: EventStream::new(),
            tick_interval: tokio::time::interval(Duration::from_millis(tick_rate_ms)),
            api_rx,
        }
    }

    pub async fn next(&mut self) -> Result<Event> {
        tokio::select! {
            // Terminal input events
            event = self.crossterm_events.next() => {
                match event {
                    Some(Ok(e)) => Ok(Event::Input(e)),
                    Some(Err(e)) => Err(e.into()),
                    None => Err(anyhow::anyhow!("Terminal event stream ended")),
                }
            }
            // Tick timer
            _ = self.tick_interval.tick() => {
                Ok(Event::Tick)
            }
            // API events from background tasks
            api_event = self.api_rx.recv() => {
                match api_event {
                    Some(e) => Ok(Event::Api(e)),
                    None => Err(anyhow::anyhow!("API event channel closed")),
                }
            }
        }
    }

    /// Non-blocking drain of any queued API events
    pub fn try_next_api(&mut self) -> Option<ApiEvent> {
        self.api_rx.try_recv().ok()
    }
}
