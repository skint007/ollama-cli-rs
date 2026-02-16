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
    let popup_height = 46.min(area.height.saturating_sub(4));
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
        Line::from("  Alt+M       Model picker"),
        Line::from("  Esc         Switch to scroll mode"),
        Line::from("  Tab         Focus sidebar"),
        Line::from(""),
        Line::from(Span::styled("Chat Scroll Mode", t.heading)),
        Line::from("  j/k         Scroll up/down"),
        Line::from("  Ctrl+D/U    Page down/up"),
        Line::from("  G           Scroll to bottom"),
        Line::from("  i/Enter     Back to input"),
        Line::from("  n           New chat"),
        Line::from("  Esc         Focus sidebar"),
        Line::from(""),
        Line::from(Span::styled("Models", t.heading)),
        Line::from("  j/k         Navigate rows"),
        Line::from("  Enter       Show model details"),
        Line::from("  p           Pull new model"),
        Line::from("  c           Copy selected model"),
        Line::from("  d           Delete model"),
        Line::from("  r           Refresh list"),
        Line::from(""),
        Line::from(Span::styled("Running", t.heading)),
        Line::from("  j/k         Navigate rows"),
        Line::from("  u           Unload from memory"),
        Line::from("  r           Refresh list"),
        Line::from(""),
        Line::from(Span::styled("Library", t.heading)),
        Line::from("  j/k         Navigate models"),
        Line::from("  Enter/p     Pull selected model"),
        Line::from("  /           Search/filter"),
        Line::from("  s           Toggle sort (popular/newest)"),
        Line::from("  r           Refresh list"),
        Line::from(""),
        Line::from(Span::styled("Config", t.heading)),
        Line::from("  j/k         Navigate profiles"),
        Line::from("  Enter       Switch to profile"),
        Line::from("  a           Add new profile"),
        Line::from("  d           Remove profile"),
        Line::from("  t           Test connection"),
        Line::from(""),
        Line::from(Span::styled("Press ? or Esc to close", t.muted)),
    ];

    let paragraph = Paragraph::new(lines)
        .block(block)
        .wrap(Wrap { trim: false });

    frame.render_widget(paragraph, popup_area);
}
