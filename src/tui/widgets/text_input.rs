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

    let Some(input_state) = &app.text_input else {
        return;
    };

    let popup_width = 60.min(area.width.saturating_sub(4));
    let popup_height = 8.min(area.height.saturating_sub(4));
    let x = (area.width.saturating_sub(popup_width)) / 2 + area.x;
    let y = (area.height.saturating_sub(popup_height)) / 2 + area.y;
    let popup_area = Rect::new(x, y, popup_width, popup_height);

    frame.render_widget(Clear, popup_area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::new().fg(Color::Cyan))
        .title(format!(" {} ", input_state.title))
        .title_style(t.title);

    // Show blinking cursor
    let cursor_char = if app.tick_count % 2 == 0 { "_" } else { " " };

    let lines = vec![
        Line::from(""),
        Line::from(Span::raw(&input_state.prompt)),
        Line::from(""),
        Line::from(vec![
            Span::styled("  > ", Style::new().fg(Color::Cyan)),
            Span::raw(&input_state.input),
            Span::styled(cursor_char, Style::new().add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  Enter", Style::new().fg(Color::Green).add_modifier(Modifier::BOLD)),
            Span::raw(" = Submit    "),
            Span::styled("Esc", Style::new().fg(Color::Red).add_modifier(Modifier::BOLD)),
            Span::raw(" = Cancel"),
        ]),
    ];

    let paragraph = Paragraph::new(lines)
        .block(block)
        .wrap(Wrap { trim: false });

    frame.render_widget(paragraph, popup_area);
}
