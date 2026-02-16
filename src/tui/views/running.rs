use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    widgets::{Block, Borders, Cell, Row, Table},
    Frame,
};

use crate::format;
use crate::tui::app::App;
use crate::tui::theme::theme;

pub fn render(frame: &mut Frame, area: Rect, app: &mut App) {
    let t = theme();

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(t.border)
        .title(" Running Models ")
        .title_style(t.title);

    if app.running.loading {
        let loading = ratatui::widgets::Paragraph::new("Loading...")
            .style(t.muted)
            .block(block);
        frame.render_widget(loading, area);
        return;
    }

    if app.running.models.is_empty() {
        let empty = ratatui::widgets::Paragraph::new("No models currently loaded in memory.")
            .style(t.muted)
            .block(block);
        frame.render_widget(empty, area);
        return;
    }

    let header = Row::new(vec![
        Cell::from("Name"),
        Cell::from("Size"),
        Cell::from("Expires"),
    ])
    .style(t.title)
    .bottom_margin(1);

    let rows: Vec<Row> = app
        .running
        .models
        .iter()
        .map(|model| {
            Row::new(vec![
                Cell::from(model.name.clone()),
                Cell::from(format::bytes_to_human(model.size)),
                Cell::from(model.expires_at.clone().unwrap_or_default()),
            ])
        })
        .collect();

    let widths = [
        ratatui::layout::Constraint::Percentage(40),
        ratatui::layout::Constraint::Percentage(25),
        ratatui::layout::Constraint::Percentage(35),
    ];

    let table = Table::new(rows, widths)
        .header(header)
        .block(block)
        .row_highlight_style(
            Style::default()
                .add_modifier(Modifier::REVERSED)
                .fg(ratatui::style::Color::Cyan),
        );

    frame.render_stateful_widget(table, area, &mut app.running.table_state);
}
