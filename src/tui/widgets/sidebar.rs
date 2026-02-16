use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem},
    Frame,
};

use crate::tui::app::{App, Focus, Section};
use crate::tui::theme::theme;

pub fn render(frame: &mut Frame, area: Rect, app: &mut App) {
    let t = theme();

    let border_style = if app.focus == Focus::Sidebar {
        t.border_focused
    } else {
        t.border
    };

    let block = Block::default()
        .borders(Borders::RIGHT)
        .border_style(border_style)
        .title(" ollama-cli ")
        .title_style(t.title);

    let items: Vec<ListItem> = Section::ALL
        .iter()
        .enumerate()
        .map(|(idx, section)| {
            let is_active = *section == app.active_section;
            let prefix = if is_active { "> " } else { "  " };
            let label = format!("{}{} {}", prefix, idx + 1, section.label());

            let style = if is_active {
                t.sidebar_selected
            } else {
                t.sidebar_unselected
            };

            ListItem::new(Line::from(Span::styled(label, style)))
        })
        .collect();

    let list = List::new(items)
        .block(block)
        .highlight_style(
            Style::default()
                .add_modifier(Modifier::REVERSED)
                .fg(ratatui::style::Color::Cyan),
        );

    frame.render_stateful_widget(list, area, &mut app.sidebar_state);
}
