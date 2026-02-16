use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Row, Table},
    Frame,
};

use crate::tui::app::{App, Focus};
use crate::tui::theme::theme;

pub fn render(frame: &mut Frame, area: Rect, app: &mut App) {
    let t = theme();
    let is_focused = app.focus == Focus::MainPanel;

    let border_style = if is_focused {
        t.border_focused
    } else {
        t.border
    };

    let title = if app.benchmark.running {
        if let Some(model) = &app.benchmark.current_model {
            format!(
                " Benchmarks - Running: {} ({}/{}) ",
                model, app.benchmark.current_round, app.benchmark.total_rounds
            )
        } else {
            " Benchmarks - Running... ".to_string()
        }
    } else {
        " Benchmarks ".to_string()
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(border_style)
        .title(title)
        .title_style(if is_focused { t.title } else { t.muted });

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if app.benchmark.available_models.is_empty() {
        let msg = Paragraph::new(vec![
            Line::from(""),
            Line::from("  No models available."),
            Line::from(""),
            Line::from(Span::styled(
                "  Pull models first, then switch to this view.",
                Style::new().fg(Color::DarkGray),
            )),
        ])
        .style(t.muted);
        frame.render_widget(msg, inner);
        return;
    }

    // Layout: model selector (left) + results (right)
    let horizontal = Layout::horizontal([
        Constraint::Length(35), // model selector
        Constraint::Min(1),    // results table
    ])
    .split(inner);

    render_model_selector(frame, horizontal[0], app);
    render_results(frame, horizontal[1], app);
}

fn render_model_selector(frame: &mut Frame, area: Rect, app: &mut App) {
    let t = theme();

    let chunks = Layout::vertical([
        Constraint::Length(2), // header + rounds
        Constraint::Min(1),   // model list
        Constraint::Length(1), // footer hint
    ])
    .split(area);

    // Header with rounds info
    let selected_count = app
        .benchmark
        .selected_models
        .iter()
        .filter(|s| **s)
        .count();

    let header = Line::from(vec![
        Span::styled(" Select Models ", t.heading),
        Span::styled(
            format!("({} selected, {} round{})", selected_count, app.benchmark.rounds,
                if app.benchmark.rounds == 1 { "" } else { "s" }),
            t.muted,
        ),
    ]);
    frame.render_widget(Paragraph::new(header), chunks[0]);

    // Model checklist
    let items: Vec<ListItem> = app
        .benchmark
        .available_models
        .iter()
        .enumerate()
        .map(|(i, name)| {
            let checked = app
                .benchmark
                .selected_models
                .get(i)
                .copied()
                .unwrap_or(false);
            let checkbox = if checked { "[x] " } else { "[ ] " };
            let style = if checked {
                Style::new().fg(Color::Green)
            } else {
                Style::default()
            };
            ListItem::new(Line::from(vec![
                Span::styled(checkbox, style),
                Span::styled(name.as_str(), style),
            ]))
        })
        .collect();

    let list = List::new(items)
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("> ");

    frame.render_stateful_widget(list, chunks[1], &mut app.benchmark.list_state);

    // Footer hint
    let hint = Line::from(Span::styled(
        " Space=Toggle  a=All  c=Clear",
        Style::new().fg(Color::DarkGray),
    ));
    frame.render_widget(Paragraph::new(hint), chunks[2]);
}

fn render_results(frame: &mut Frame, area: Rect, app: &App) {
    let t = theme();

    if app.benchmark.results.is_empty() && !app.benchmark.running {
        let msg = Paragraph::new(vec![
            Line::from(""),
            Line::from(""),
            Line::from(Span::styled(
                "  Select models and press Enter to run benchmark",
                Style::new().fg(Color::DarkGray),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "  Each model will generate a response and report",
                Style::new().fg(Color::DarkGray),
            )),
            Line::from(Span::styled(
                "  tokens/sec, token count, and response time.",
                Style::new().fg(Color::DarkGray),
            )),
        ])
        .style(t.muted);
        frame.render_widget(msg, area);
        return;
    }

    let chunks = Layout::vertical([
        Constraint::Length(1), // title
        Constraint::Min(1),   // table
        Constraint::Length(2), // winner + status
    ])
    .split(area);

    // Title
    let title = Line::from(Span::styled(" Results", t.heading));
    frame.render_widget(Paragraph::new(title), chunks[0]);

    // Results table
    let header = Row::new(vec!["MODEL", "TOKENS/SEC", "AVG TOKENS", "AVG TIME", "STATUS"])
        .style(
            Style::new()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        );

    let rows: Vec<Row> = app
        .benchmark
        .results
        .iter()
        .map(|r| {
            let status = if r.successful_rounds == r.total_rounds {
                Span::styled("OK", Style::new().fg(Color::Green))
            } else if r.successful_rounds > 0 {
                Span::styled("PARTIAL", Style::new().fg(Color::Yellow))
            } else {
                Span::styled("FAILED", Style::new().fg(Color::Red))
            };

            Row::new(vec![
                r.name.clone(),
                if r.successful_rounds > 0 {
                    format!("{:.2}", r.avg_speed)
                } else {
                    "N/A".to_string()
                },
                if r.successful_rounds > 0 {
                    r.avg_tokens.to_string()
                } else {
                    "N/A".to_string()
                },
                if r.successful_rounds > 0 {
                    format!("{:.2}s", r.avg_time)
                } else {
                    "N/A".to_string()
                },
                format!("{}", status),
            ])
        })
        .collect();

    let widths = [
        Constraint::Min(15),
        Constraint::Length(12),
        Constraint::Length(12),
        Constraint::Length(10),
        Constraint::Length(8),
    ];

    let table = Table::new(rows, widths)
        .header(header)
        .column_spacing(1);

    frame.render_widget(table, chunks[1]);

    // Winner + running status
    let mut footer_lines = Vec::new();

    if app.benchmark.running {
        if let Some(model) = &app.benchmark.current_model {
            let spinner = match (app.tick_count / 2) % 4 {
                0 => "|",
                1 => "/",
                2 => "-",
                _ => "\\",
            };
            footer_lines.push(Line::from(vec![
                Span::styled(
                    format!(" {} Running: {} ", spinner, model),
                    Style::new().fg(Color::Yellow),
                ),
                Span::styled(
                    format!(
                        "(round {}/{})",
                        app.benchmark.current_round, app.benchmark.total_rounds
                    ),
                    t.muted,
                ),
            ]));
        }
    } else if !app.benchmark.results.is_empty() {
        let winner = app
            .benchmark
            .results
            .iter()
            .filter(|r| r.successful_rounds > 0)
            .max_by(|a, b| a.avg_speed.partial_cmp(&b.avg_speed).unwrap_or(std::cmp::Ordering::Equal));

        if let Some(w) = winner {
            footer_lines.push(Line::from(vec![
                Span::styled(" Winner: ", Style::new().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::styled(
                    format!("{} ({:.2} tokens/sec)", w.name, w.avg_speed),
                    Style::new().fg(Color::Green).add_modifier(Modifier::BOLD),
                ),
            ]));
        }
    }

    frame.render_widget(Paragraph::new(footer_lines), chunks[2]);
}
