use ratatui::{
    layout::Rect,
    widgets::{Block, Borders},
    Frame,
};

use crate::tui::app::{App, ChatFocus};
use crate::tui::theme::theme;

pub fn render(frame: &mut Frame, area: Rect, app: &mut App, is_focused: bool) {
    let t = theme();

    let input_active = is_focused && app.chat.chat_focus == ChatFocus::Input;

    let border_style = if input_active {
        t.border_focused
    } else {
        t.border
    };

    let title = if app.chat.is_streaming {
        " Streaming... "
    } else if app.chat.chat_focus == ChatFocus::Messages {
        " Input (i=focus) "
    } else {
        " Input (Enter=Send, Alt+Enter=Newline) "
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(border_style)
        .title(title)
        .title_style(if input_active { t.title } else { t.muted });

    app.chat.textarea.set_block(block);

    if app.chat.is_streaming || app.chat.chat_focus == ChatFocus::Messages {
        app.chat
            .textarea
            .set_cursor_style(ratatui::style::Style::default());
    } else if is_focused {
        app.chat.textarea.set_cursor_style(
            ratatui::style::Style::default()
                .fg(ratatui::style::Color::Cyan)
                .add_modifier(ratatui::style::Modifier::REVERSED),
        );
    } else {
        app.chat
            .textarea
            .set_cursor_style(ratatui::style::Style::default());
    }

    frame.render_widget(&app.chat.textarea, area);
}
