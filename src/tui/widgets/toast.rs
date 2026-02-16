use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};

use crate::tui::app::App;

pub fn render(frame: &mut Frame, area: Rect, app: &App) {
    let Some((msg, _)) = &app.status_message else {
        return;
    };

    let border_color = if msg.contains("FAILED") || msg.contains("error") || msg.contains("Error")
    {
        Color::Red
    } else if msg.contains("OK") || msg.contains("success") || msg.contains("Switched") {
        Color::Green
    } else {
        Color::Yellow
    };

    let text_style = Style::new().fg(Color::White);

    // Size the popup to fit the message
    let msg_width = msg.len() as u16 + 4; // padding
    let popup_width = msg_width.clamp(20, 60).min(area.width.saturating_sub(4));
    let popup_height = 3_u16.min(area.height.saturating_sub(2));

    // Position: bottom-right, just above the status bar
    let x = area.x + area.width.saturating_sub(popup_width + 2);
    let y = area.y + area.height.saturating_sub(popup_height + 2); // +2 to sit above status bar

    let popup_area = Rect::new(x, y, popup_width, popup_height);

    frame.render_widget(Clear, popup_area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::new().fg(border_color))
        .style(Style::new().bg(Color::Rgb(30, 30, 30)));

    let icon = match border_color {
        Color::Red => Span::styled("✗ ", Style::new().fg(Color::Red).add_modifier(Modifier::BOLD)),
        Color::Green => {
            Span::styled("✓ ", Style::new().fg(Color::Green).add_modifier(Modifier::BOLD))
        }
        _ => Span::styled("● ", Style::new().fg(Color::Yellow)),
    };

    let content = Line::from(vec![icon, Span::styled(msg.as_str(), text_style)]);

    let paragraph = Paragraph::new(content)
        .block(block)
        .wrap(Wrap { trim: false });

    frame.render_widget(paragraph, popup_area);
}
