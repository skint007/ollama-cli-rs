use ratatui::{
    layout::Rect,
    widgets::{Block, Borders},
    Frame,
};

use crate::tui::app::App;
use crate::tui::theme::theme;

pub fn render(frame: &mut Frame, area: Rect, app: &mut App, is_focused: bool) {
    let t = theme();

    let border_style = if is_focused {
        t.border_focused
    } else {
        t.border
    };

    let title = if app.chat.is_streaming {
        " Streaming... "
    } else {
        " Input (Enter=Send, Alt+Enter=Newline) "
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(border_style)
        .title(title)
        .title_style(if is_focused { t.title } else { t.muted });

    app.chat.textarea.set_block(block);

    if app.chat.is_streaming {
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
