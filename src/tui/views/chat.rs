use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::Modifier,
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use crate::tui::app::{App, Focus};
use crate::tui::theme::theme;
use crate::tui::widgets::{input_box, message_list};

pub fn render(frame: &mut Frame, area: Rect, app: &mut App) {
    let t = theme();

    let chunks = Layout::vertical([
        Constraint::Length(1),  // header
        Constraint::Min(1),    // messages
        Constraint::Length(5), // input
    ])
    .split(area);

    // Header
    let model_name = app
        .chat
        .current_model
        .as_deref()
        .unwrap_or("No model selected");

    let speed_str = match app.chat.last_tokens_per_sec {
        Some(tps) if !app.chat.is_streaming => format!("  {:.1} tok/s", tps),
        _ => String::new(),
    };

    let header = Line::from(vec![
        Span::styled(" Chat - ", t.title),
        Span::styled(model_name, t.title.add_modifier(Modifier::ITALIC)),
        Span::styled(speed_str, t.muted),
        Span::raw("  "),
        Span::styled("Alt+M=Model", t.muted),
    ]);

    frame.render_widget(Paragraph::new(header), chunks[0]);

    // Messages
    message_list::render(frame, chunks[1], app);

    // Input
    let is_focused = app.focus == Focus::MainPanel && !app.chat.is_streaming;
    input_box::render(frame, chunks[2], app, is_focused);
}
