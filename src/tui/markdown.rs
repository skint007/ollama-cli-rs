use pulldown_cmark::{Event, Parser, Tag, TagEnd};
use ratatui::{
    style::{Color, Style},
    text::{Line, Span},
};

use super::theme::theme;

pub fn to_lines(text: &str, _width: u16) -> Vec<Line<'static>> {
    let t = theme();
    let parser = Parser::new(text);
    let mut lines: Vec<Line<'static>> = Vec::new();
    let mut current_spans: Vec<Span<'static>> = Vec::new();
    let mut style_stack: Vec<Style> = vec![Style::default()];
    let mut in_code_block = false;
    let mut list_depth: usize = 0;
    let mut ordered_index: Option<u64> = None;

    for event in parser {
        match event {
            Event::Start(Tag::Heading { level, .. }) => {
                flush_line(&mut lines, &mut current_spans);
                let style = match level {
                    pulldown_cmark::HeadingLevel::H1 => t.heading,
                    _ => t.heading2,
                };
                style_stack.push(style);
            }
            Event::End(TagEnd::Heading(_)) => {
                style_stack.pop();
                flush_line(&mut lines, &mut current_spans);
            }
            Event::Start(Tag::Strong) => {
                style_stack.push(merge_style(current_style(&style_stack), t.bold));
            }
            Event::End(TagEnd::Strong) => {
                style_stack.pop();
            }
            Event::Start(Tag::Emphasis) => {
                style_stack.push(merge_style(current_style(&style_stack), t.italic));
            }
            Event::End(TagEnd::Emphasis) => {
                style_stack.pop();
            }
            Event::Start(Tag::Link { .. }) => {
                style_stack.push(t.link);
            }
            Event::End(TagEnd::Link) => {
                style_stack.pop();
            }
            Event::Start(Tag::BlockQuote(_)) => {
                flush_line(&mut lines, &mut current_spans);
                style_stack.push(t.blockquote);
            }
            Event::End(TagEnd::BlockQuote(_)) => {
                style_stack.pop();
                flush_line(&mut lines, &mut current_spans);
            }
            Event::Start(Tag::List(start)) => {
                flush_line(&mut lines, &mut current_spans);
                list_depth += 1;
                ordered_index = start;
            }
            Event::End(TagEnd::List(_)) => {
                list_depth = list_depth.saturating_sub(1);
                ordered_index = None;
                flush_line(&mut lines, &mut current_spans);
            }
            Event::Start(Tag::Item) => {
                flush_line(&mut lines, &mut current_spans);
                let indent = "  ".repeat(list_depth.saturating_sub(1));
                let bullet = if let Some(idx) = &mut ordered_index {
                    let b = format!("{}{}. ", indent, idx);
                    *idx += 1;
                    b
                } else {
                    format!("{}- ", indent)
                };
                current_spans.push(Span::styled(bullet, t.list_bullet));
            }
            Event::End(TagEnd::Item) => {
                flush_line(&mut lines, &mut current_spans);
            }
            Event::Start(Tag::CodeBlock(kind)) => {
                flush_line(&mut lines, &mut current_spans);
                in_code_block = true;
                // Add language label if present
                if let pulldown_cmark::CodeBlockKind::Fenced(lang) = kind {
                    let lang = lang.to_string();
                    if !lang.is_empty() {
                        lines.push(Line::from(Span::styled(
                            format!("[{}]", lang),
                            Style::default().fg(Color::DarkGray),
                        )));
                    }
                }
            }
            Event::End(TagEnd::CodeBlock) => {
                flush_line(&mut lines, &mut current_spans);
                in_code_block = false;
            }
            Event::Start(Tag::Paragraph) => {
                // No special handling needed
            }
            Event::End(TagEnd::Paragraph) => {
                flush_line(&mut lines, &mut current_spans);
                lines.push(Line::from(""));
            }
            Event::Code(text) => {
                current_spans.push(Span::styled(text.to_string(), t.code_inline));
            }
            Event::Text(text) => {
                let style = current_style(&style_stack);
                if in_code_block {
                    // Code blocks: render each line with code style
                    let code_style = Style::default().fg(Color::White).bg(t.code_block_bg);
                    for (i, line) in text.lines().enumerate() {
                        if i > 0 {
                            flush_line(&mut lines, &mut current_spans);
                        }
                        current_spans.push(Span::styled(line.to_string(), code_style));
                    }
                } else {
                    current_spans.push(Span::styled(text.to_string(), style));
                }
            }
            Event::SoftBreak => {
                current_spans.push(Span::raw(" "));
            }
            Event::HardBreak => {
                flush_line(&mut lines, &mut current_spans);
            }
            Event::Rule => {
                flush_line(&mut lines, &mut current_spans);
                lines.push(Line::from(Span::styled(
                    "\u{2500}".repeat(40),
                    Style::default().fg(Color::DarkGray),
                )));
            }
            _ => {}
        }
    }

    // Flush remaining
    if !current_spans.is_empty() {
        lines.push(Line::from(std::mem::take(&mut current_spans)));
    }

    // Remove trailing empty lines
    while lines.last().map_or(false, |l| l.spans.is_empty() || (l.spans.len() == 1 && l.spans[0].content.is_empty())) {
        lines.pop();
    }

    lines
}

fn flush_line(lines: &mut Vec<Line<'static>>, spans: &mut Vec<Span<'static>>) {
    if !spans.is_empty() {
        lines.push(Line::from(std::mem::take(spans)));
    }
}

fn current_style(stack: &[Style]) -> Style {
    stack.last().copied().unwrap_or_default()
}

fn merge_style(base: Style, overlay: Style) -> Style {
    let mut result = base;
    if let Some(fg) = overlay.fg {
        result = result.fg(fg);
    }
    if let Some(bg) = overlay.bg {
        result = result.bg(bg);
    }
    result = result.add_modifier(overlay.add_modifier);
    result
}
