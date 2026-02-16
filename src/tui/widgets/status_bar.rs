use ratatui::{
    layout::{Alignment, Rect},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use crate::tui::app::{App, Focus, Section};
use crate::tui::theme::theme;

pub fn render(frame: &mut Frame, area: Rect, app: &App) {
    let t = theme();

    let connected_indicator = if app.connected {
        Span::styled(" Connected ", t.status_connected)
    } else {
        Span::styled(" Disconnected ", t.status_disconnected)
    };

    let url = Span::styled(format!(" {} ", app.client.url()), t.status_bar);

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
    let left =
        Line::from(vec![connected_indicator, url, model_info, status_msg]).style(t.status_bar);

    let left_para = Paragraph::new(left).style(t.status_bar);

    // Right side: context-aware keybinding hints
    let hints = context_hints(app);
    let right = Paragraph::new(Line::from(hints.as_str()))
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

fn context_hints(app: &App) -> String {
    if app.show_help {
        return " ?/Esc=Close ".to_string();
    }
    if app.confirm.is_some() {
        return " y=Yes  n/Esc=Cancel ".to_string();
    }
    if app.text_input.is_some() {
        return " Enter=Submit  Esc=Cancel ".to_string();
    }
    if app.pull.is_some() {
        if app.pull.as_ref().is_some_and(|p| p.is_complete) {
            return " Esc=Close ".to_string();
        }
        return " Pulling... ".to_string();
    }
    if app.model_detail.is_some() {
        return " j/k=Scroll  Esc=Close ".to_string();
    }
    if app.show_model_picker {
        return " Enter=Select  Esc=Cancel ".to_string();
    }

    match app.focus {
        Focus::Sidebar => " j/k=Nav  Enter=Select  ?=Help ".to_string(),
        Focus::MainPanel => match app.active_section {
            Section::Chat => {
                use crate::tui::app::ChatFocus;
                if app.chat.is_streaming {
                    " j/k=Scroll  G=Bottom  ?=Help ".to_string()
                } else if app.chat.chat_focus == ChatFocus::Messages {
                    " j/k=Scroll  i=Input  Esc=Sidebar  ?=Help ".to_string()
                } else {
                    " Enter=Send  Alt+M=Model  Esc=Scroll  ?=Help ".to_string()
                }
            }
            Section::Models => {
                " j/k=Nav  Enter=Detail  p=Pull  c=Copy  d=Delete  ?=Help ".to_string()
            }
            Section::Running => " j/k=Nav  u=Unload  r=Refresh  ?=Help ".to_string(),
            _ => " ?=Help  q=Quit  Tab=Nav ".to_string(),
        },
    }
}
