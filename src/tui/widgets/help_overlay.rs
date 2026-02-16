use ratatui::{
    layout::Rect,
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};

use crate::tui::app::App;
use crate::tui::theme::theme;

pub fn render(frame: &mut Frame, area: Rect, _app: &App) {
    let t = theme();

    // Center the popup
    let popup_width = 56.min(area.width.saturating_sub(4));
    let popup_height = 28.min(area.height.saturating_sub(4));
    let x = (area.width.saturating_sub(popup_width)) / 2 + area.x;
    let y = (area.height.saturating_sub(popup_height)) / 2 + area.y;
    let popup_area = Rect::new(x, y, popup_width, popup_height);

    frame.render_widget(Clear, popup_area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(t.border_focused)
        .title(" Help - Key Bindings ")
        .title_style(t.title);

    let lines = vec![
        Line::from(Span::styled("Global", t.heading)),
        Line::from("  q           Quit"),
        Line::from("  ?           Toggle this help"),
        Line::from("  Tab         Focus sidebar"),
        Line::from("  1-6         Jump to section"),
        Line::from("  Ctrl+C      Force quit"),
        Line::from(""),
        Line::from(Span::styled("Sidebar", t.heading)),
        Line::from("  j/k         Navigate up/down"),
        Line::from("  Enter/l     Select section"),
        Line::from(""),
        Line::from(Span::styled("Chat Input", t.heading)),
        Line::from("  Enter       Send message"),
        Line::from("  Alt+Enter   New line"),
        Line::from("  Ctrl+M      Model picker"),
        Line::from("  Esc         Focus sidebar"),
        Line::from(""),
        Line::from(Span::styled("Chat Messages (while streaming)", t.heading)),
        Line::from("  j/k         Scroll up/down"),
        Line::from("  Ctrl+D/U    Page down/up"),
        Line::from("  G           Scroll to bottom"),
        Line::from("  n           New chat"),
        Line::from(""),
        Line::from(Span::styled("Models / Running", t.heading)),
        Line::from("  j/k         Navigate"),
        Line::from("  r           Refresh"),
        Line::from("  Enter       Show details (models)"),
        Line::from(""),
        Line::from(Span::styled("Press ? or Esc to close", t.muted)),
    ];

    let paragraph = Paragraph::new(lines)
        .block(block)
        .wrap(Wrap { trim: false });

    frame.render_widget(paragraph, popup_area);
}
