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
        .title(" Models ")
        .title_style(t.title);

    if app.models.loading {
        let loading = ratatui::widgets::Paragraph::new("Loading models...")
            .style(t.muted)
            .block(block);
        frame.render_widget(loading, area);
        return;
    }

    if app.models.models.is_empty() {
        let empty = ratatui::widgets::Paragraph::new("No models found. Pull a model first.")
            .style(t.muted)
            .block(block);
        frame.render_widget(empty, area);
        return;
    }

    let header = Row::new(vec![
        Cell::from("Name"),
        Cell::from("Size"),
        Cell::from("Family"),
        Cell::from("Params"),
        Cell::from("Quant"),
    ])
    .style(t.title)
    .bottom_margin(1);

    let rows: Vec<Row> = app
        .models
        .models
        .iter()
        .map(|model| {
            let details = model.details.as_ref();
            Row::new(vec![
                Cell::from(model.name.clone()),
                Cell::from(format::bytes_to_human(model.size)),
                Cell::from(
                    details
                        .and_then(|d| d.family.clone())
                        .unwrap_or_default(),
                ),
                Cell::from(
                    details
                        .and_then(|d| d.parameter_size.clone())
                        .unwrap_or_default(),
                ),
                Cell::from(
                    details
                        .and_then(|d| d.quantization_level.clone())
                        .unwrap_or_default(),
                ),
            ])
        })
        .collect();

    let widths = [
        ratatui::layout::Constraint::Percentage(30),
        ratatui::layout::Constraint::Percentage(15),
        ratatui::layout::Constraint::Percentage(20),
        ratatui::layout::Constraint::Percentage(15),
        ratatui::layout::Constraint::Percentage(20),
    ];

    let table = Table::new(rows, widths)
        .header(header)
        .block(block)
        .row_highlight_style(
            Style::default()
                .add_modifier(Modifier::REVERSED)
                .fg(ratatui::style::Color::Cyan),
        );

    frame.render_stateful_widget(table, area, &mut app.models.table_state);
}
