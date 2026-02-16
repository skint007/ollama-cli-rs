use ratatui::{
    layout::{Constraint, Layout},
    Frame,
};

use super::app::{App, Section};
use super::views;
use super::widgets::{help_overlay, model_picker, sidebar, status_bar};

pub fn render(frame: &mut Frame, app: &mut App) {
    let area = frame.area();

    // Vertical split: body + status bar
    let vertical = Layout::vertical([
        Constraint::Min(1),    // body
        Constraint::Length(1), // status bar
    ])
    .split(area);

    // Horizontal split: sidebar + main panel
    let horizontal = Layout::horizontal([
        Constraint::Length(20), // sidebar
        Constraint::Min(1),    // main panel
    ])
    .split(vertical[0]);

    // Render sidebar
    sidebar::render(frame, horizontal[0], app);

    // Render main panel based on active section
    match app.active_section {
        Section::Chat => views::chat::render(frame, horizontal[1], app),
        Section::Models => views::models::render(frame, horizontal[1], app),
        Section::Running => views::running::render(frame, horizontal[1], app),
        Section::Library => views::library::render(frame, horizontal[1], app),
        Section::Benchmarks => views::benchmark::render(frame, horizontal[1], app),
        Section::Config => views::config::render(frame, horizontal[1], app),
    }

    // Render status bar
    status_bar::render(frame, vertical[1], app);

    // Render overlays on top
    if app.show_model_picker {
        model_picker::render(frame, area, app);
    }

    if app.show_help {
        help_overlay::render(frame, area, app);
    }
}
