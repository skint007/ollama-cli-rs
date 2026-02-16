use ratatui::widgets::{ListState, TableState};
use std::time::Instant;
use tokio::sync::mpsc;
use tui_textarea::TextArea;

use crate::api::types::{
    ChatMessage, ChatRequest, ChatResponse, DeleteRequest, GenerateRequest, ModelInfo, PsResponse,
    RunningModel, ShowRequest, ShowResponse, TagsResponse,
};
use crate::client::OllamaClient;
use crate::config::Config;

use super::action::Action;
use super::event::ApiEvent;
use super::keymap::{map_key_event, InputContext};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Section {
    Chat,
    Models,
    Running,
    Library,
    Benchmarks,
    Config,
}

impl Section {
    pub const ALL: [Section; 6] = [
        Section::Chat,
        Section::Models,
        Section::Running,
        Section::Library,
        Section::Benchmarks,
        Section::Config,
    ];

    pub fn label(&self) -> &'static str {
        match self {
            Section::Chat => "Chat",
            Section::Models => "Models",
            Section::Running => "Running",
            Section::Library => "Library",
            Section::Benchmarks => "Benchmarks",
            Section::Config => "Config",
        }
    }

    pub fn index(&self) -> usize {
        Section::ALL.iter().position(|s| s == self).unwrap_or(0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Focus {
    Sidebar,
    MainPanel,
}

pub struct ChatState {
    pub messages: Vec<ChatMessage>,
    pub current_model: Option<String>,
    pub textarea: TextArea<'static>,
    pub scroll_offset: usize,
    pub is_streaming: bool,
    pub streaming_buffer: String,
    pub last_stats: Option<ChatStats>,
    pub auto_scroll: bool,
}

pub struct ChatStats {
    pub eval_count: u64,
    pub eval_duration: u64,
    pub total_duration: u64,
}

pub struct ModelsState {
    pub models: Vec<ModelInfo>,
    pub table_state: TableState,
    pub loading: bool,
}

pub struct RunningState {
    pub models: Vec<RunningModel>,
    pub table_state: TableState,
    pub loading: bool,
}

pub struct ModelPickerState {
    pub filter: String,
    pub filtered_models: Vec<String>,
    pub list_state: ListState,
}

/// State for the model detail overlay
pub struct ModelDetailState {
    pub name: String,
    pub response: ShowResponse,
    pub scroll_offset: u16,
}

/// State for a confirmation dialog
pub struct ConfirmState {
    pub title: String,
    pub message: String,
    pub on_confirm: ConfirmAction,
}

#[derive(Clone)]
pub enum ConfirmAction {
    DeleteModel(String),
    UnloadModel(String),
}

pub struct App {
    pub client: OllamaClient,
    pub config: Config,
    pub api_tx: mpsc::UnboundedSender<ApiEvent>,
    pub should_quit: bool,

    pub active_section: Section,
    pub sidebar_state: ListState,
    pub focus: Focus,

    pub show_help: bool,
    pub show_model_picker: bool,
    pub model_picker: ModelPickerState,
    pub model_detail: Option<ModelDetailState>,
    pub confirm: Option<ConfirmState>,

    pub chat: ChatState,
    pub models: ModelsState,
    pub running: RunningState,

    pub status_message: Option<(String, Instant)>,
    pub connected: bool,
    pub tick_count: u64,
}

impl App {
    pub fn new(
        client: OllamaClient,
        config: Config,
        api_tx: mpsc::UnboundedSender<ApiEvent>,
    ) -> Self {
        let mut sidebar_state = ListState::default();
        sidebar_state.select(Some(0));

        let mut textarea = TextArea::default();
        textarea.set_placeholder_text("Type your message...");

        Self {
            client,
            config,
            api_tx,
            should_quit: false,

            active_section: Section::Chat,
            sidebar_state,
            focus: Focus::MainPanel,

            show_help: false,
            show_model_picker: false,
            model_picker: ModelPickerState {
                filter: String::new(),
                filtered_models: Vec::new(),
                list_state: ListState::default(),
            },
            model_detail: None,
            confirm: None,

            chat: ChatState {
                messages: Vec::new(),
                current_model: None,
                textarea,
                scroll_offset: 0,
                is_streaming: false,
                streaming_buffer: String::new(),
                last_stats: None,
                auto_scroll: true,
            },

            models: ModelsState {
                models: Vec::new(),
                table_state: TableState::default(),
                loading: false,
            },

            running: RunningState {
                models: Vec::new(),
                table_state: TableState::default(),
                loading: false,
            },

            status_message: None,
            connected: false,
            tick_count: 0,
        }
    }

    pub fn on_tick(&mut self) {
        self.tick_count = self.tick_count.wrapping_add(1);

        // Clear stale status messages (after 5 seconds)
        if let Some((_, created)) = &self.status_message {
            if created.elapsed() > std::time::Duration::from_secs(5) {
                self.status_message = None;
            }
        }
    }

    pub fn handle_input(&mut self, event: crossterm::event::Event) -> Action {
        use crossterm::event::Event as CE;

        match event {
            CE::Key(key) => self.handle_key(key),
            CE::Resize(_, _) => Action::None,
            _ => Action::None,
        }
    }

    fn handle_key(&mut self, key: crossterm::event::KeyEvent) -> Action {
        use crossterm::event::KeyCode;

        // Help overlay takes priority
        if self.show_help {
            if matches!(key.code, KeyCode::Char('?') | KeyCode::Esc) {
                self.show_help = false;
            }
            return Action::None;
        }

        // Confirmation dialog takes priority
        if self.confirm.is_some() {
            return self.handle_confirm_key(key);
        }

        // Model detail overlay takes priority
        if self.model_detail.is_some() {
            return self.handle_model_detail_key(key);
        }

        // Model picker overlay takes priority
        if self.show_model_picker {
            return self.handle_model_picker_key(key);
        }

        // Determine input context and map to action
        let context = self.current_input_context();
        let action = map_key_event(key, &context);
        self.execute_action(action.clone());
        action
    }

    fn current_input_context(&self) -> InputContext {
        match self.focus {
            Focus::Sidebar => InputContext::Sidebar,
            Focus::MainPanel => match self.active_section {
                Section::Chat => {
                    if self.chat.is_streaming {
                        InputContext::ChatMessages
                    } else {
                        InputContext::ChatInput
                    }
                }
                Section::Models => InputContext::ModelsTable,
                Section::Running => InputContext::RunningTable,
                _ => InputContext::Global,
            },
        }
    }

    fn execute_action(&mut self, action: Action) {
        match action {
            Action::Quit => self.should_quit = true,
            Action::ToggleHelp => self.show_help = !self.show_help,
            Action::FocusSidebar => self.focus = Focus::Sidebar,
            Action::FocusMainPanel => self.focus = Focus::MainPanel,
            Action::NavigateUp => self.navigate(-1),
            Action::NavigateDown => self.navigate(1),
            Action::SelectSection => self.select_sidebar_section(),
            Action::JumpToSection(section) => {
                let prev = self.active_section;
                self.active_section = section;
                self.sidebar_state.select(Some(section.index()));
                self.focus = Focus::MainPanel;
                self.on_section_switch(prev, section);
            }

            // Chat actions
            Action::SendMessage => self.send_chat_message(),
            Action::ToggleModelPicker => self.toggle_model_picker(),
            Action::ScrollUp => self.scroll_chat(-1),
            Action::ScrollDown => self.scroll_chat(1),
            Action::ScrollPageUp => self.scroll_chat(-20),
            Action::ScrollPageDown => self.scroll_chat(20),
            Action::ScrollToBottom => {
                self.chat.scroll_offset = 0;
                self.chat.auto_scroll = true;
            }
            Action::NewChat => {
                self.chat.messages.clear();
                self.chat.streaming_buffer.clear();
                self.chat.scroll_offset = 0;
                self.chat.last_stats = None;
                self.chat.auto_scroll = true;
            }
            Action::FocusChatInput => {
                self.focus = Focus::MainPanel;
            }

            // Models
            Action::RefreshModels => self.fetch_models(),
            Action::ShowModelDetail => self.show_model_detail(),
            Action::DeleteModel => self.confirm_delete_model(),

            // Running
            Action::RefreshRunning => self.fetch_running(),
            Action::UnloadModel => self.confirm_unload_model(),

            // Confirmation
            Action::ConfirmYes | Action::ConfirmNo => {}

            // Close overlay
            Action::CloseOverlay => {
                self.model_detail = None;
                self.confirm = None;
            }

            // Model picker handled separately
            Action::PickerConfirm | Action::PickerCancel => {}

            Action::None => {}
        }
    }

    /// Navigate up/down based on current focus and section
    fn navigate(&mut self, delta: i32) {
        match self.focus {
            Focus::Sidebar => self.navigate_sidebar(delta),
            Focus::MainPanel => match self.active_section {
                Section::Models => self.navigate_models_table(delta),
                Section::Running => self.navigate_running_table(delta),
                _ => {}
            },
        }
    }

    fn navigate_sidebar(&mut self, delta: i32) {
        let len = Section::ALL.len();
        let current = self.sidebar_state.selected().unwrap_or(0) as i32;
        let next = (current + delta).rem_euclid(len as i32) as usize;
        self.sidebar_state.select(Some(next));
    }

    fn navigate_models_table(&mut self, delta: i32) {
        let len = self.models.models.len();
        if len == 0 {
            return;
        }
        let current = self.models.table_state.selected().unwrap_or(0) as i32;
        let next = (current + delta).rem_euclid(len as i32) as usize;
        self.models.table_state.select(Some(next));
    }

    fn navigate_running_table(&mut self, delta: i32) {
        let len = self.running.models.len();
        if len == 0 {
            return;
        }
        let current = self.running.table_state.selected().unwrap_or(0) as i32;
        let next = (current + delta).rem_euclid(len as i32) as usize;
        self.running.table_state.select(Some(next));
    }

    fn select_sidebar_section(&mut self) {
        if let Some(idx) = self.sidebar_state.selected() {
            if idx < Section::ALL.len() {
                let prev = self.active_section;
                let next = Section::ALL[idx];
                self.active_section = next;
                self.focus = Focus::MainPanel;
                self.on_section_switch(prev, next);
            }
        }
    }

    /// Auto-refresh data when switching to a section
    fn on_section_switch(&mut self, _prev: Section, next: Section) {
        match next {
            Section::Models => self.fetch_models(),
            Section::Running => self.fetch_running(),
            _ => {}
        }
    }

    fn scroll_chat(&mut self, delta: i32) {
        self.chat.auto_scroll = false;
        let current = self.chat.scroll_offset as i32;
        self.chat.scroll_offset = (current - delta).max(0) as usize;
    }

    fn toggle_model_picker(&mut self) {
        self.show_model_picker = !self.show_model_picker;
        if self.show_model_picker {
            self.model_picker.filter.clear();
            self.update_model_picker_filter();
            self.model_picker.list_state.select(Some(0));
        }
    }

    fn update_model_picker_filter(&mut self) {
        let filter = self.model_picker.filter.to_lowercase();
        self.model_picker.filtered_models = self
            .models
            .models
            .iter()
            .map(|m| m.name.clone())
            .filter(|name| filter.is_empty() || name.to_lowercase().contains(&filter))
            .collect();
    }

    fn handle_model_picker_key(&mut self, key: crossterm::event::KeyEvent) -> Action {
        use crossterm::event::KeyCode;

        match key.code {
            KeyCode::Esc => {
                self.show_model_picker = false;
                Action::PickerCancel
            }
            KeyCode::Enter => {
                if let Some(idx) = self.model_picker.list_state.selected() {
                    if let Some(model) = self.model_picker.filtered_models.get(idx) {
                        self.chat.current_model = Some(model.clone());
                    }
                }
                self.show_model_picker = false;
                Action::PickerConfirm
            }
            KeyCode::Up => {
                let len = self.model_picker.filtered_models.len();
                if len > 0 {
                    let current =
                        self.model_picker.list_state.selected().unwrap_or(0) as i32;
                    let next = (current - 1).rem_euclid(len as i32) as usize;
                    self.model_picker.list_state.select(Some(next));
                }
                Action::None
            }
            KeyCode::Down => {
                let len = self.model_picker.filtered_models.len();
                if len > 0 {
                    let current =
                        self.model_picker.list_state.selected().unwrap_or(0) as i32;
                    let next = (current + 1).rem_euclid(len as i32) as usize;
                    self.model_picker.list_state.select(Some(next));
                }
                Action::None
            }
            KeyCode::Backspace => {
                self.model_picker.filter.pop();
                self.update_model_picker_filter();
                if !self.model_picker.filtered_models.is_empty() {
                    self.model_picker.list_state.select(Some(0));
                }
                Action::None
            }
            KeyCode::Char(c) => {
                self.model_picker.filter.push(c);
                self.update_model_picker_filter();
                if !self.model_picker.filtered_models.is_empty() {
                    self.model_picker.list_state.select(Some(0));
                }
                Action::None
            }
            _ => Action::None,
        }
    }

    fn handle_confirm_key(&mut self, key: crossterm::event::KeyEvent) -> Action {
        use crossterm::event::KeyCode;

        match key.code {
            KeyCode::Char('y') | KeyCode::Char('Y') => {
                if let Some(state) = self.confirm.take() {
                    self.execute_confirmed(state.on_confirm);
                }
                Action::ConfirmYes
            }
            KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                self.confirm = None;
                Action::ConfirmNo
            }
            _ => Action::None,
        }
    }

    fn handle_model_detail_key(&mut self, key: crossterm::event::KeyEvent) -> Action {
        use crossterm::event::KeyCode;

        match key.code {
            KeyCode::Esc | KeyCode::Char('q') => {
                self.model_detail = None;
                Action::CloseOverlay
            }
            KeyCode::Char('j') | KeyCode::Down => {
                if let Some(detail) = &mut self.model_detail {
                    detail.scroll_offset = detail.scroll_offset.saturating_add(1);
                }
                Action::None
            }
            KeyCode::Char('k') | KeyCode::Up => {
                if let Some(detail) = &mut self.model_detail {
                    detail.scroll_offset = detail.scroll_offset.saturating_sub(1);
                }
                Action::None
            }
            KeyCode::Char('d')
                if key
                    .modifiers
                    .contains(crossterm::event::KeyModifiers::CONTROL) =>
            {
                if let Some(detail) = &mut self.model_detail {
                    detail.scroll_offset = detail.scroll_offset.saturating_add(10);
                }
                Action::None
            }
            KeyCode::Char('u')
                if key
                    .modifiers
                    .contains(crossterm::event::KeyModifiers::CONTROL) =>
            {
                if let Some(detail) = &mut self.model_detail {
                    detail.scroll_offset = detail.scroll_offset.saturating_sub(10);
                }
                Action::None
            }
            _ => Action::None,
        }
    }

    // === Model actions ===

    fn show_model_detail(&mut self) {
        let name = if let Some(idx) = self.models.table_state.selected() {
            self.models.models.get(idx).map(|m| m.name.clone())
        } else {
            None
        };

        let Some(name) = name else {
            self.status_message = Some(("No model selected".to_string(), Instant::now()));
            return;
        };

        let client = self.client.clone();
        let tx = self.api_tx.clone();
        let model_name = name.clone();

        tokio::spawn(async move {
            let request = ShowRequest {
                name: model_name.clone(),
                verbose: false,
            };
            match client
                .post::<ShowRequest, ShowResponse>("/api/show", &request)
                .await
            {
                Ok(resp) => {
                    let _ = tx.send(ApiEvent::ModelDetailLoaded {
                        name: model_name,
                        response: resp,
                    });
                }
                Err(e) => {
                    let _ = tx.send(ApiEvent::Error(format!(
                        "Failed to load model details: {}",
                        e
                    )));
                }
            }
        });

        self.status_message = Some((format!("Loading details for {}...", name), Instant::now()));
    }

    fn confirm_delete_model(&mut self) {
        let name = if let Some(idx) = self.models.table_state.selected() {
            self.models.models.get(idx).map(|m| m.name.clone())
        } else {
            None
        };

        let Some(name) = name else {
            self.status_message = Some(("No model selected".to_string(), Instant::now()));
            return;
        };

        self.confirm = Some(ConfirmState {
            title: "Delete Model".to_string(),
            message: format!("Delete model '{}'? This cannot be undone. (y/n)", name),
            on_confirm: ConfirmAction::DeleteModel(name),
        });
    }

    fn confirm_unload_model(&mut self) {
        let name = if let Some(idx) = self.running.table_state.selected() {
            self.running.models.get(idx).map(|m| m.name.clone())
        } else {
            None
        };

        let Some(name) = name else {
            self.status_message = Some(("No model selected".to_string(), Instant::now()));
            return;
        };

        self.confirm = Some(ConfirmState {
            title: "Unload Model".to_string(),
            message: format!(
                "Unload '{}' from memory? It can be loaded again later. (y/n)",
                name
            ),
            on_confirm: ConfirmAction::UnloadModel(name),
        });
    }

    fn execute_confirmed(&mut self, action: ConfirmAction) {
        match action {
            ConfirmAction::DeleteModel(name) => self.delete_model(&name),
            ConfirmAction::UnloadModel(name) => self.unload_model(&name),
        }
    }

    fn delete_model(&mut self, name: &str) {
        let client = self.client.clone();
        let tx = self.api_tx.clone();
        let model_name = name.to_string();

        tokio::spawn(async move {
            let request = DeleteRequest {
                name: model_name.clone(),
            };
            match client.delete("/api/delete", &request).await {
                Ok(resp) => {
                    if resp.status().is_success() {
                        let _ = tx.send(ApiEvent::ModelDeleted(model_name));
                    } else {
                        let status = resp.status();
                        let body = resp.text().await.unwrap_or_default();
                        let _ = tx.send(ApiEvent::Error(format!(
                            "Failed to delete model (HTTP {}): {}",
                            status, body
                        )));
                    }
                }
                Err(e) => {
                    let _ = tx.send(ApiEvent::Error(format!("Failed to delete model: {}", e)));
                }
            }
        });

        self.status_message = Some((format!("Deleting {}...", name), Instant::now()));
    }

    fn unload_model(&mut self, name: &str) {
        let client = self.client.clone();
        let tx = self.api_tx.clone();
        let model_name = name.to_string();

        tokio::spawn(async move {
            let request = GenerateRequest {
                model: model_name.clone(),
                prompt: String::new(),
                stream: false,
                keep_alive: Some(0),
                ..Default::default()
            };
            match client.post_raw("/api/generate", &request).await {
                Ok(resp) => {
                    if resp.status().is_success() {
                        let _ = tx.send(ApiEvent::ModelUnloaded(model_name));
                    } else {
                        let status = resp.status();
                        let body = resp.text().await.unwrap_or_default();
                        let _ = tx.send(ApiEvent::Error(format!(
                            "Failed to unload model (HTTP {}): {}",
                            status, body
                        )));
                    }
                }
                Err(e) => {
                    let _ = tx.send(ApiEvent::Error(format!("Failed to unload model: {}", e)));
                }
            }
        });

        self.status_message = Some((format!("Unloading {}...", name), Instant::now()));
    }

    // === Chat ===

    pub fn send_chat_message(&mut self) {
        let input = self.chat.textarea.lines().join("\n");
        let input = input.trim().to_string();
        if input.is_empty() {
            return;
        }

        let model = match &self.chat.current_model {
            Some(m) => m.clone(),
            None => {
                self.status_message = Some((
                    "No model selected. Press Ctrl+M to pick one.".to_string(),
                    Instant::now(),
                ));
                return;
            }
        };

        // Add user message
        self.chat.messages.push(ChatMessage {
            role: "user".to_string(),
            content: input,
        });

        // Clear input
        self.chat.textarea = TextArea::default();
        self.chat
            .textarea
            .set_placeholder_text("Type your message...");
        self.chat.is_streaming = true;
        self.chat.streaming_buffer.clear();
        self.chat.auto_scroll = true;

        // Build request
        let request = ChatRequest {
            model,
            messages: self.chat.messages.clone(),
            stream: true,
            format: None,
            options: None,
        };

        let client = self.client.clone();
        let tx = self.api_tx.clone();

        tokio::spawn(async move {
            match client.post_stream("/api/chat", &request).await {
                Ok(mut stream) => {
                    while let Ok(Some(chunk)) = stream.next_json::<ChatResponse>().await {
                        if let Some(err) = &chunk.error {
                            let _ = tx.send(ApiEvent::ChatError(err.clone()));
                            return;
                        }
                        if let Some(msg) = &chunk.message {
                            if !msg.content.is_empty() {
                                let _ = tx.send(ApiEvent::ChatToken(msg.content.clone()));
                            }
                        }
                        if chunk.done == Some(true) {
                            let _ = tx.send(ApiEvent::ChatDone {
                                eval_count: chunk.eval_count,
                                eval_duration: chunk.eval_duration,
                                total_duration: chunk.total_duration,
                            });
                            return;
                        }
                    }
                }
                Err(e) => {
                    let _ = tx.send(ApiEvent::ChatError(e.to_string()));
                }
            }
        });
    }

    // === API event handling ===

    pub fn handle_api_event(&mut self, event: ApiEvent) {
        match event {
            ApiEvent::ChatToken(token) => {
                self.chat.streaming_buffer.push_str(&token);
            }
            ApiEvent::ChatDone {
                eval_count,
                eval_duration,
                total_duration,
            } => {
                if !self.chat.streaming_buffer.is_empty() {
                    self.chat.messages.push(ChatMessage {
                        role: "assistant".to_string(),
                        content: std::mem::take(&mut self.chat.streaming_buffer),
                    });
                }
                self.chat.is_streaming = false;

                if let (Some(ec), Some(ed), Some(td)) = (eval_count, eval_duration, total_duration)
                {
                    self.chat.last_stats = Some(ChatStats {
                        eval_count: ec,
                        eval_duration: ed,
                        total_duration: td,
                    });
                }
            }
            ApiEvent::ChatError(err) => {
                self.chat.is_streaming = false;
                self.chat.streaming_buffer.clear();
                self.status_message = Some((format!("Chat error: {}", err), Instant::now()));
            }
            ApiEvent::ModelsLoaded(models) => {
                self.models.models = models;
                self.models.loading = false;
                // Select first row if nothing selected
                if self.models.table_state.selected().is_none() && !self.models.models.is_empty() {
                    self.models.table_state.select(Some(0));
                }
                // Auto-select first model for chat if none selected
                if self.chat.current_model.is_none() {
                    if let Some(first) = self.models.models.first() {
                        self.chat.current_model = Some(first.name.clone());
                    }
                }
            }
            ApiEvent::RunningModelsLoaded(models) => {
                self.running.models = models;
                self.running.loading = false;
                if self.running.table_state.selected().is_none()
                    && !self.running.models.is_empty()
                {
                    self.running.table_state.select(Some(0));
                }
            }
            ApiEvent::ModelDetailLoaded { name, response } => {
                self.model_detail = Some(ModelDetailState {
                    name,
                    response,
                    scroll_offset: 0,
                });
                self.status_message = None;
            }
            ApiEvent::ModelDeleted(name) => {
                self.status_message =
                    Some((format!("Model '{}' deleted", name), Instant::now()));
                self.fetch_models();
            }
            ApiEvent::ModelUnloaded(name) => {
                self.status_message =
                    Some((format!("Model '{}' unloaded", name), Instant::now()));
                self.fetch_running();
            }
            ApiEvent::ConnectionStatus(connected) => {
                self.connected = connected;
            }
            ApiEvent::Error(err) => {
                self.status_message = Some((err, Instant::now()));
            }
        }
    }

    // === Data fetching ===

    pub fn fetch_models(&mut self) {
        self.models.loading = true;
        let client = self.client.clone();
        let tx = self.api_tx.clone();

        tokio::spawn(async move {
            match client.get::<TagsResponse>("/api/tags").await {
                Ok(resp) => {
                    let _ = tx.send(ApiEvent::ModelsLoaded(resp.models));
                }
                Err(e) => {
                    let _ = tx.send(ApiEvent::Error(format!("Failed to fetch models: {}", e)));
                }
            }
        });
    }

    pub fn fetch_running(&mut self) {
        self.running.loading = true;
        let client = self.client.clone();
        let tx = self.api_tx.clone();

        tokio::spawn(async move {
            match client.get::<PsResponse>("/api/ps").await {
                Ok(resp) => {
                    let _ = tx.send(ApiEvent::RunningModelsLoaded(resp.models));
                }
                Err(e) => {
                    let _ = tx.send(ApiEvent::Error(format!(
                        "Failed to fetch running models: {}",
                        e
                    )));
                }
            }
        });
    }

    pub fn test_connection(&mut self) {
        let client = self.client.clone();
        let tx = self.api_tx.clone();

        tokio::spawn(async move {
            let connected = client.test_connection().await.is_ok();
            let _ = tx.send(ApiEvent::ConnectionStatus(connected));
        });
    }
}
