use ratatui::{
    layout::{Alignment, Rect},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use crate::tui::app::{App, Section};
use crate::tui::theme::theme;

pub fn render(frame: &mut Frame, area: Rect, app: &App) {
    let t = theme();

    let connected_indicator = if app.connected {
        Span::styled(" Connected ", t.status_connected)
    } else {
        Span::styled(" Disconnected ", t.status_disconnected)
    };

    let url = Span::styled(
        format!(" {} ", app.client.url()),
        t.status_bar,
    );

    let model_info = if app.active_section == Section::Chat {
        if let Some(model) = &app.chat.current_model {
            Span::styled(format!(" Model: {} ", model), t.status_bar)
        } else {
            Span::styled(" No model ", t.status_bar)
        }
    } else {
        Span::raw("")
    };

    let status_msg = if let Some((msg, _)) = &app.status_message {
        Span::styled(format!(" {} ", msg), t.status_bar)
    } else {
        Span::raw("")
    };

    // Left side: connection + url + model + status
    let left = Line::from(vec![connected_indicator, url, model_info, status_msg])
        .style(t.status_bar);

    let left_para = Paragraph::new(left).style(t.status_bar);

    // Right side: keybinding hints
    let hints = " ?=Help  q=Quit  Tab=Nav ";
    let right = Paragraph::new(Line::from(hints))
        .style(t.status_bar)
        .alignment(Alignment::Right);

    // Render the status bar background
    let bg = Paragraph::new("").style(t.status_bar);
    frame.render_widget(bg, area);

    // Split area for left and right
    let left_width = area.width.saturating_sub(hints.len() as u16);
    let left_area = Rect {
        width: left_width,
        ..area
    };
    let right_area = Rect {
        x: area.x + left_width,
        width: area.width - left_width,
        ..area
    };

    frame.render_widget(left_para, left_area);
    frame.render_widget(right, right_area);
}
