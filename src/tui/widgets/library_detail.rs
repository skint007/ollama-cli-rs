use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
    Frame,
};

use crate::tui::app::App;
use crate::tui::theme::theme;

pub fn render(frame: &mut Frame, area: Rect, app: &mut App) {
    let Some(detail) = &app.library_detail else {
        return;
    };
    let t = theme();

    let popup_width = (area.width * 4 / 5).min(area.width.saturating_sub(4));
    let popup_height = (area.height * 4 / 5).min(area.height.saturating_sub(4));
    let x = (area.width.saturating_sub(popup_width)) / 2 + area.x;
    let y = (area.height.saturating_sub(popup_height)) / 2 + area.y;
    let popup_area = Rect::new(x, y, popup_width, popup_height);

    frame.render_widget(Clear, popup_area);

    let outer_block = Block::default()
        .borders(Borders::ALL)
        .border_style(t.border_focused)
        .title(format!(" {} ", detail.model.name))
        .title_style(t.title);

    let inner = outer_block.inner(popup_area);
    frame.render_widget(outer_block, popup_area);

    // Build description + capabilities lines
    let mut info_lines: Vec<Line> = Vec::new();
    if !detail.model.description.is_empty() {
        info_lines.push(Line::from(detail.model.description.clone()));
        info_lines.push(Line::from(""));
    }
    if !detail.model.capabilities.is_empty() {
        info_lines.push(Line::from(vec![
            Span::styled("Capabilities: ", Style::new().add_modifier(Modifier::BOLD)),
            Span::styled(
                detail.model.capabilities.join("  "),
                Style::new().fg(Color::Green),
            ),
        ]));
        info_lines.push(Line::from(""));
    }

    // Clamp info height so the tag list always has room
    let info_height = (info_lines.len() as u16).min(inner.height.saturating_sub(5));
    let tags_title = if detail.loading { " Tags (loading…) " } else { " Tags " };

    let chunks = Layout::vertical([
        Constraint::Length(info_height),
        Constraint::Min(3),
        Constraint::Length(1),
    ])
    .split(inner);

    // Info pane
    let info_para = Paragraph::new(info_lines).wrap(Wrap { trim: false });
    frame.render_widget(info_para, chunks[0]);

    // Tags pane
    let tags_block = Block::default()
        .borders(Borders::TOP)
        .border_style(t.muted)
        .title(tags_title)
        .title_style(t.muted);

    let tags_inner = tags_block.inner(chunks[1]);
    frame.render_widget(tags_block, chunks[1]);

    let tag_width = tags_inner.width.saturating_sub(4) as usize; // leave room for "> "
    let items: Vec<ListItem> = detail
        .tags
        .iter()
        .map(|tag| {
            let name_col = 30.min(tag_width / 2);
            if tag.size.is_empty() {
                ListItem::new(Line::from(Span::raw(tag.name.clone())))
            } else {
                let pad = tag_width.saturating_sub(name_col + tag.size.len() + 2);
                ListItem::new(Line::from(vec![
                    Span::raw(format!("{:<width$}", tag.name, width = name_col)),
                    Span::raw(" ".repeat(pad)),
                    Span::styled(tag.size.clone(), Style::new().fg(Color::Yellow)),
                ]))
            }
        })
        .collect();

    let list = List::new(items)
        .highlight_style(
            Style::new()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("> ");

    frame.render_stateful_widget(list, tags_inner, &mut app.library_detail.as_mut().unwrap().list_state);

    // Footer
    let footer = Line::from(vec![
        Span::styled("Enter/p", Style::new().fg(Color::Green).add_modifier(Modifier::BOLD)),
        Span::raw("=Pull  "),
        Span::styled("j/k", Style::new().fg(Color::Cyan)),
        Span::raw("=Navigate  "),
        Span::styled("Esc", Style::new().fg(Color::Red).add_modifier(Modifier::BOLD)),
        Span::raw("=Close"),
    ]);
    frame.render_widget(Paragraph::new(footer), chunks[2]);
}
