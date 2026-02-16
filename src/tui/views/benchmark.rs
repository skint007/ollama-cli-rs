use ratatui::{
    layout::{Alignment, Rect},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::tui::app::App;
use crate::tui::theme::theme;

pub fn render(frame: &mut Frame, area: Rect, _app: &mut App) {
    let t = theme();

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(t.border)
        .title(" Benchmarks ")
        .title_style(t.title);

    let content = Paragraph::new("Benchmark runner coming soon...")
        .style(t.muted)
        .alignment(Alignment::Center)
        .block(block);

    frame.render_widget(content, area);
}
