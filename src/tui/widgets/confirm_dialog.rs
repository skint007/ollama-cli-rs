use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};

use crate::tui::app::App;
use crate::tui::theme::theme;

pub fn render(frame: &mut Frame, area: Rect, app: &App) {
    let t = theme();

    let Some(confirm) = &app.confirm else {
        return;
    };

    let popup_width = 50.min(area.width.saturating_sub(4));
    let popup_height = 7.min(area.height.saturating_sub(4));
    let x = (area.width.saturating_sub(popup_width)) / 2 + area.x;
    let y = (area.height.saturating_sub(popup_height)) / 2 + area.y;
    let popup_area = Rect::new(x, y, popup_width, popup_height);

    frame.render_widget(Clear, popup_area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::new().fg(Color::Yellow))
        .title(format!(" {} ", confirm.title))
        .title_style(t.title);

    let lines = vec![
        Line::from(""),
        Line::from(Span::raw(&confirm.message)),
        Line::from(""),
        Line::from(vec![
            Span::styled("  y", Style::new().fg(Color::Green).add_modifier(Modifier::BOLD)),
            Span::raw(" = Yes    "),
            Span::styled("n", Style::new().fg(Color::Red).add_modifier(Modifier::BOLD)),
            Span::raw("/"),
            Span::styled("Esc", Style::new().fg(Color::Red).add_modifier(Modifier::BOLD)),
            Span::raw(" = Cancel"),
        ]),
    ];

    let paragraph = Paragraph::new(lines)
        .block(block)
        .wrap(Wrap { trim: false });

    frame.render_widget(paragraph, popup_area);
}
