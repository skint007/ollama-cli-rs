use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

use crate::tui::app::{App, Focus, LibrarySort};
use crate::tui::theme::theme;

pub fn render(frame: &mut Frame, area: Rect, app: &mut App) {
    let t = theme();
    let is_focused = app.focus == Focus::MainPanel;

    let border_style = if is_focused {
        t.border_focused
    } else {
        t.border
    };

    let sort_label = match app.library.sort {
        LibrarySort::Popular => "Popular",
        LibrarySort::Newest => "Newest",
    };

    let title = if app.library.search.is_empty() {
        format!(" Library - ollama.com ({}) ", sort_label)
    } else {
        format!(
            " Library - ollama.com ({}) [filter: {}] ",
            sort_label, app.library.search
        )
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(border_style)
        .title(title)
        .title_style(if is_focused { t.title } else { t.muted });

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if app.library.loading {
        let loading = Paragraph::new(vec![
            Line::from(""),
            Line::from("  Fetching models from ollama.com..."),
        ])
        .style(t.muted);
        frame.render_widget(loading, inner);
        return;
    }

    if app.library.models.is_empty() {
        let msg = Paragraph::new(vec![
            Line::from(""),
            Line::from("  No models loaded."),
            Line::from(""),
            Line::from(Span::styled(
                "  Press 'r' to fetch from ollama.com",
                Style::new().fg(Color::DarkGray),
            )),
        ])
        .style(t.muted);
        frame.render_widget(msg, inner);
        return;
    }

    if app.library.filtered.is_empty() {
        let msg = Paragraph::new(vec![
            Line::from(""),
            Line::from(format!(
                "  No models matching '{}'",
                app.library.search
            )),
            Line::from(""),
            Line::from(Span::styled(
                "  Press '/' to change search filter",
                Style::new().fg(Color::DarkGray),
            )),
        ])
        .style(t.muted);
        frame.render_widget(msg, inner);
        return;
    }

    // Layout: model count header + list
    let chunks = Layout::vertical([
        Constraint::Length(1), // count
        Constraint::Min(1),   // list
    ])
    .split(inner);

    let count_line = Line::from(vec![
        Span::styled(
            format!(
                " {} model(s)",
                app.library.filtered.len()
            ),
            t.muted,
        ),
        Span::styled(
            format!("  (of {} total)", app.library.models.len()),
            t.muted,
        ),
    ]);
    frame.render_widget(Paragraph::new(count_line), chunks[0]);

    let items: Vec<ListItem> = app
        .library
        .filtered
        .iter()
        .filter_map(|&idx| app.library.models.get(idx))
        .map(|model| {
            let mut spans = vec![Span::styled(
                &model.name,
                Style::new()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )];

            if !model.tags.is_empty() {
                spans.push(Span::raw("  "));
                spans.push(Span::styled(
                    format!("[{}]", model.tags.join(" ")),
                    Style::new().fg(Color::Yellow),
                ));
            }

            if !model.capabilities.is_empty() {
                spans.push(Span::raw("  "));
                spans.push(Span::styled(
                    model.capabilities.join(" "),
                    Style::new().fg(Color::Green),
                ));
            }

            let name_line = Line::from(spans);

            let desc_line = if model.description.is_empty() {
                Line::from(Span::styled("  (no description)", t.muted))
            } else {
                let max_len = (area.width as usize).saturating_sub(6);
                let desc = if model.description.len() > max_len {
                    format!("  {}...", &model.description[..max_len.saturating_sub(3)])
                } else {
                    format!("  {}", &model.description)
                };
                Line::from(Span::styled(desc, Style::new().fg(Color::DarkGray)))
            };

            ListItem::new(vec![name_line, desc_line])
        })
        .collect();

    let list = List::new(items)
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("> ");

    frame.render_stateful_widget(list, chunks[1], &mut app.library.list_state);
}
