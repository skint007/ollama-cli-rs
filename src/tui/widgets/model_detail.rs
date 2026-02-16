use ratatui::{
    layout::Rect,
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};

use crate::tui::app::App;
use crate::tui::theme::theme;

pub fn render(frame: &mut Frame, area: Rect, app: &App) {
    let t = theme();

    let Some(detail) = &app.model_detail else {
        return;
    };

    // Center the popup (large - 80% of screen)
    let popup_width = (area.width * 4 / 5).min(area.width.saturating_sub(4));
    let popup_height = (area.height * 4 / 5).min(area.height.saturating_sub(4));
    let x = (area.width.saturating_sub(popup_width)) / 2 + area.x;
    let y = (area.height.saturating_sub(popup_height)) / 2 + area.y;
    let popup_area = Rect::new(x, y, popup_width, popup_height);

    frame.render_widget(Clear, popup_area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(t.border_focused)
        .title(format!(" Model: {} ", detail.name))
        .title_style(t.title);

    let mut lines: Vec<Line> = Vec::new();

    // Parameters
    if let Some(params) = &detail.response.parameters {
        lines.push(Line::from(Span::styled("Parameters", t.heading)));
        for line in params.lines() {
            lines.push(Line::from(format!("  {}", line)));
        }
        lines.push(Line::from(""));
    }

    // Template
    if let Some(template) = &detail.response.template {
        lines.push(Line::from(Span::styled("Template", t.heading)));
        for line in template.lines() {
            lines.push(Line::from(format!("  {}", line)));
        }
        lines.push(Line::from(""));
    }

    // Model info (JSON)
    if let Some(info) = &detail.response.model_info {
        lines.push(Line::from(Span::styled("Model Info", t.heading)));
        if let Ok(pretty) = serde_json::to_string_pretty(info) {
            for line in pretty.lines() {
                lines.push(Line::from(format!("  {}", line)));
            }
        }
        lines.push(Line::from(""));
    }

    // Modelfile
    if let Some(modelfile) = &detail.response.modelfile {
        lines.push(Line::from(Span::styled("Modelfile", t.heading)));
        for line in modelfile.lines() {
            lines.push(Line::from(format!("  {}", line)));
        }
        lines.push(Line::from(""));
    }

    // Footer
    lines.push(Line::from(Span::styled(
        "j/k=Scroll  Ctrl+D/U=Page  Esc=Close",
        t.muted,
    )));

    let paragraph = Paragraph::new(lines)
        .block(block)
        .wrap(Wrap { trim: false })
        .scroll((detail.scroll_offset, 0));

    frame.render_widget(paragraph, popup_area);
}
