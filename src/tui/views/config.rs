use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

use crate::tui::app::{App, Focus};
use crate::tui::theme::theme;

pub fn render(frame: &mut Frame, area: Rect, app: &mut App) {
    let t = theme();
    let is_focused = app.focus == Focus::MainPanel;

    let border_style = if is_focused {
        t.border_focused
    } else {
        t.border
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(border_style)
        .title(" Config - URL Profiles ")
        .title_style(if is_focused { t.title } else { t.muted });

    let inner = block.inner(area);
    frame.render_widget(block, area);

    // Layout: info section + profile list
    let chunks = Layout::vertical([
        Constraint::Length(5), // current info
        Constraint::Min(1),   // profile list
    ])
    .split(inner);

    render_info(frame, chunks[0], app);
    render_profiles(frame, chunks[1], app);
}

fn render_info(frame: &mut Frame, area: Rect, app: &App) {
    let t = theme();

    let active = app
        .config
        .active_profile
        .as_deref()
        .unwrap_or("(default)");

    let active_url = app.config.resolve_url(None);

    let conn_status = if app.connected {
        Span::styled("Connected", Style::new().fg(Color::Green))
    } else {
        Span::styled("Disconnected", Style::new().fg(Color::Red))
    };

    let lines = vec![
        Line::from(vec![
            Span::styled("  Active:  ", Style::new().add_modifier(Modifier::BOLD)),
            Span::raw(active),
        ]),
        Line::from(vec![
            Span::styled("  URL:     ", Style::new().add_modifier(Modifier::BOLD)),
            Span::raw(active_url),
        ]),
        Line::from(vec![
            Span::styled("  Status:  ", Style::new().add_modifier(Modifier::BOLD)),
            conn_status,
        ]),
        Line::from(""),
        Line::from(Span::styled("  Profiles:", t.heading)),
    ];

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, area);
}

fn render_profiles(frame: &mut Frame, area: Rect, app: &mut App) {
    let t = theme();

    if app.config_state.profiles.is_empty() {
        let msg = Paragraph::new(vec![
            Line::from(""),
            Line::from("  No profiles configured."),
            Line::from(""),
            Line::from(Span::styled(
                "  Press 'a' to add a profile.",
                Style::new().fg(Color::DarkGray),
            )),
        ])
        .style(t.muted);
        frame.render_widget(msg, area);
        return;
    }

    let active = app.config.active_profile.as_deref();

    let items: Vec<ListItem> = app
        .config_state
        .profiles
        .iter()
        .map(|(name, url)| {
            let is_active = active == Some(name.as_str());
            let indicator = if is_active { "* " } else { "  " };
            let name_style = if is_active {
                Style::new()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            let line = Line::from(vec![
                Span::styled(indicator, name_style),
                Span::styled(name, name_style),
                Span::styled("  ", Style::default()),
                Span::styled(url, Style::new().fg(Color::DarkGray)),
            ]);
            ListItem::new(line)
        })
        .collect();

    let list = List::new(items)
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("> ");

    frame.render_stateful_widget(list, area, &mut app.config_state.list_state);
}
