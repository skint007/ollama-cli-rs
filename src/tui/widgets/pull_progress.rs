use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};

use crate::format;
use crate::tui::app::App;
use crate::tui::theme::theme;

pub fn render(frame: &mut Frame, area: Rect, app: &App) {
    let t = theme();

    let Some(pull) = &app.pull else {
        return;
    };

    // Size the popup based on content
    let layer_count = pull.layer_order.len();
    let content_height = 4 + layer_count + 3; // status + layers + footer
    let popup_width = 70.min(area.width.saturating_sub(4));
    let popup_height = (content_height as u16 + 2).min(area.height.saturating_sub(4));
    let x = (area.width.saturating_sub(popup_width)) / 2 + area.x;
    let y = (area.height.saturating_sub(popup_height)) / 2 + area.y;
    let popup_area = Rect::new(x, y, popup_width, popup_height);

    frame.render_widget(Clear, popup_area);

    let border_color = if pull.is_complete {
        if pull.status == "success" {
            Color::Green
        } else {
            Color::Red
        }
    } else {
        Color::Cyan
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::new().fg(border_color))
        .title(format!(" Pulling: {} ", pull.model_name))
        .title_style(t.title);

    let mut lines = Vec::new();
    lines.push(Line::from(""));

    // Status line
    let status_style = if pull.is_complete && pull.status == "success" {
        Style::new().fg(Color::Green)
    } else if pull.is_complete {
        Style::new().fg(Color::Red)
    } else {
        Style::new().fg(Color::Yellow)
    };

    let spinner = if !pull.is_complete {
        let frames = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
        let idx = app.tick_count as usize % frames.len();
        format!("{} ", frames[idx])
    } else if pull.status == "success" {
        "✓ ".to_string()
    } else {
        "✗ ".to_string()
    };

    let display_status = match pull.status.as_str() {
        "pulling manifest" => "Pulling manifest...".to_string(),
        "verifying sha256 digest" => "Verifying integrity...".to_string(),
        "writing manifest" => "Writing manifest...".to_string(),
        "removing any unused layers" => "Cleaning up unused layers...".to_string(),
        "success" => "Pull complete!".to_string(),
        s if s.starts_with("pulling ") => "Downloading layers...".to_string(),
        s => s.to_string(),
    };

    lines.push(Line::from(vec![
        Span::raw("  "),
        Span::styled(spinner, status_style),
        Span::styled(display_status, status_style),
    ]));
    lines.push(Line::from(""));

    // Render each layer's progress
    let bar_width = popup_width.saturating_sub(30) as usize; // space for label + stats
    for digest in &pull.layer_order {
        if let Some(layer) = pull.layers.get(digest) {
            let short_id = if digest.len() > 15 {
                &digest[7..15]
            } else {
                digest.as_str()
            };

            let pct = if layer.total > 0 {
                (layer.completed as f64 / layer.total as f64 * 100.0) as u64
            } else {
                0
            };

            let filled = if layer.total > 0 {
                (layer.completed as f64 / layer.total as f64 * bar_width as f64) as usize
            } else {
                0
            };
            let empty = bar_width.saturating_sub(filled);

            let bar_filled = "█".repeat(filled);
            let bar_empty = "░".repeat(empty);

            let done = layer.completed >= layer.total && layer.total > 0;
            let bar_color = if done { Color::Green } else { Color::Cyan };

            let size_info = format!(
                " {}/{} {:>3}%",
                format::bytes_to_human(layer.completed),
                format::bytes_to_human(layer.total),
                pct,
            );

            lines.push(Line::from(vec![
                Span::styled(
                    format!("  {:<8} ", short_id),
                    Style::new().fg(Color::Cyan),
                ),
                Span::styled(bar_filled, Style::new().fg(bar_color)),
                Span::styled(bar_empty, Style::new().fg(Color::DarkGray)),
                Span::raw(size_info),
            ]));
        }
    }

    // Footer
    lines.push(Line::from(""));
    if pull.is_complete {
        lines.push(Line::from(vec![
            Span::raw("  "),
            Span::styled(
                "Press Esc to close",
                Style::new()
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::ITALIC),
            ),
        ]));
    }

    let paragraph = Paragraph::new(lines)
        .block(block)
        .wrap(Wrap { trim: false });

    frame.render_widget(paragraph, popup_area);
}
