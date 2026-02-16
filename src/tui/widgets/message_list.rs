use ratatui::{
    layout::Rect,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

use crate::tui::app::App;
use crate::tui::markdown;
use crate::tui::theme::theme;

pub fn render(frame: &mut Frame, area: Rect, app: &mut App) {
    let t = theme();

    let block = Block::default()
        .borders(Borders::TOP | Borders::BOTTOM)
        .border_style(t.border);

    let inner = block.inner(area);

    let mut lines: Vec<Line> = Vec::new();

    for msg in &app.chat.messages {
        match msg.role.as_str() {
            "user" => {
                lines.push(Line::from(Span::styled("You:", t.user_label)));
                // User messages rendered as plain text with wrapping
                for text_line in msg.content.lines() {
                    lines.push(Line::from(format!("  {}", text_line)));
                }
                lines.push(Line::from(""));
            }
            "assistant" => {
                lines.push(Line::from(Span::styled("Assistant:", t.assistant_label)));
                let md_lines = markdown::to_lines(&msg.content, inner.width.saturating_sub(2));
                for line in md_lines {
                    // Indent markdown content
                    let mut spans = vec![Span::raw("  ")];
                    spans.extend(line.spans);
                    lines.push(Line::from(spans));
                }
                lines.push(Line::from(""));
            }
            "system" => {
                lines.push(Line::from(Span::styled("System:", t.system_label)));
                for text_line in msg.content.lines() {
                    lines.push(Line::from(Span::styled(
                        format!("  {}", text_line),
                        t.system_label,
                    )));
                }
                lines.push(Line::from(""));
            }
            _ => {}
        }
    }

    // Render streaming buffer
    if app.chat.is_streaming && !app.chat.streaming_buffer.is_empty() {
        lines.push(Line::from(Span::styled("Assistant:", t.assistant_label)));
        let md_lines =
            markdown::to_lines(&app.chat.streaming_buffer, inner.width.saturating_sub(2));
        for line in md_lines {
            let mut spans = vec![Span::raw("  ")];
            spans.extend(line.spans);
            lines.push(Line::from(spans));
        }

        // Blinking cursor indicator
        if app.tick_count % 4 < 2 {
            let last_line = lines.last_mut();
            if let Some(line) = last_line {
                line.spans.push(Span::styled(
                    "\u{2588}",
                    ratatui::style::Style::new().fg(ratatui::style::Color::Cyan),
                ));
            }
        }
        lines.push(Line::from(""));
    } else if app.chat.is_streaming {
        lines.push(Line::from(Span::styled("Assistant:", t.assistant_label)));
        // Show thinking indicator
        let dots = match app.tick_count % 4 {
            0 => "  Thinking",
            1 => "  Thinking.",
            2 => "  Thinking..",
            _ => "  Thinking...",
        };
        lines.push(Line::from(Span::styled(dots, t.muted)));
        lines.push(Line::from(""));
    }

    if lines.is_empty() {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "  Start a conversation by typing a message below.",
            t.muted,
        )));
        if app.chat.current_model.is_none() {
            lines.push(Line::from(Span::styled(
                "  Press Ctrl+M to select a model first.",
                t.muted,
            )));
        }
    }

    // Calculate scroll - auto-scroll to bottom if enabled
    let total_lines = lines.len() as u16;
    let visible_lines = inner.height;

    let scroll = if app.chat.auto_scroll {
        total_lines.saturating_sub(visible_lines)
    } else {
        let max_scroll = total_lines.saturating_sub(visible_lines);
        (app.chat.scroll_offset as u16).min(max_scroll)
    };

    let paragraph = Paragraph::new(lines)
        .block(block)
        .wrap(Wrap { trim: false })
        .scroll((scroll, 0));

    frame.render_widget(paragraph, area);
}
