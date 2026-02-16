use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
    Frame,
};

use crate::tui::app::App;
use crate::tui::theme::theme;

pub fn render(frame: &mut Frame, area: Rect, app: &mut App) {
    let t = theme();

    // Center the popup
    let popup_width = 44.min(area.width.saturating_sub(4));
    let popup_height = 16.min(area.height.saturating_sub(4));
    let x = (area.width.saturating_sub(popup_width)) / 2 + area.x;
    let y = (area.height.saturating_sub(popup_height)) / 2 + area.y;
    let popup_area = Rect::new(x, y, popup_width, popup_height);

    // Clear the area behind the popup
    frame.render_widget(Clear, popup_area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(t.border_focused)
        .title(" Select Model ")
        .title_style(t.title);

    let inner = block.inner(popup_area);

    let chunks = Layout::vertical([
        Constraint::Length(1), // filter input
        Constraint::Length(1), // separator
        Constraint::Min(1),   // model list
        Constraint::Length(1), // help text
    ])
    .split(inner);

    // Filter input
    let filter_line = Line::from(vec![
        Span::styled("Filter: ", t.muted),
        Span::raw(&app.model_picker.filter),
        Span::styled(
            "\u{2588}",
            Style::default().fg(ratatui::style::Color::Cyan),
        ),
    ]);
    frame.render_widget(Paragraph::new(filter_line), chunks[0]);

    // Separator
    let sep = Line::from(Span::styled(
        "\u{2500}".repeat(inner.width as usize),
        t.border,
    ));
    frame.render_widget(Paragraph::new(sep), chunks[1]);

    // Model list
    let items: Vec<ListItem> = app
        .model_picker
        .filtered_models
        .iter()
        .map(|name| {
            let is_current = app.chat.current_model.as_deref() == Some(name.as_str());
            let prefix = if is_current { "* " } else { "  " };
            let style = if is_current {
                Style::default()
                    .fg(ratatui::style::Color::Green)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            ListItem::new(Line::from(Span::styled(format!("{}{}", prefix, name), style)))
        })
        .collect();

    let list = List::new(items).highlight_style(
        Style::default()
            .add_modifier(Modifier::REVERSED)
            .fg(ratatui::style::Color::Cyan),
    );

    frame.render_stateful_widget(list, chunks[2], &mut app.model_picker.list_state);

    // Help text
    let help = Line::from(Span::styled("Enter=Select  Esc=Cancel", t.muted));
    frame.render_widget(Paragraph::new(help), chunks[3]);

    // Render the block border on top
    frame.render_widget(block, popup_area);
}
