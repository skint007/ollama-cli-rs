pub mod action;
pub mod app;
pub mod event;
pub mod keymap;
pub mod markdown;
pub mod theme;
pub mod ui;
pub mod views;
pub mod widgets;

use std::io::{self, stdout};

use anyhow::Result;
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use tokio::sync::mpsc;

use crate::client::OllamaClient;
use crate::config::Config;

use app::App;
use event::{Event, EventHandler};

pub async fn run(client: OllamaClient, config: Config) -> Result<()> {
    // Setup terminal
    let mut terminal = setup_terminal()?;

    // Create channel for API events
    let (api_tx, api_rx) = mpsc::unbounded_channel();

    // Create app state
    let mut app = App::new(client, config, api_tx);

    // Create event handler
    let mut events = EventHandler::new(api_rx, 250);

    // Initial data fetches
    app.test_connection();
    app.fetch_models();

    // Main loop
    loop {
        // Render
        terminal.draw(|frame| ui::render(frame, &mut app))?;

        // Wait for next event
        let event = events.next().await?;

        match event {
            Event::Input(crossterm_event) => {
                // Check if the event should be passed to textarea first
                if should_pass_to_textarea(&app, &crossterm_event) {
                    if let crossterm::event::Event::Key(key) = crossterm_event {
                        app.chat.textarea.input(key);
                    }
                } else {
                    let action = app.handle_input(crossterm_event);
                    if action == action::Action::Quit {
                        break;
                    }
                }
            }
            Event::Tick => {
                app.on_tick();
            }
            Event::Api(api_event) => {
                app.handle_api_event(api_event);
                // Drain any additional queued API events
                while let Some(extra) = events.try_next_api() {
                    app.handle_api_event(extra);
                }
            }
        }

        if app.should_quit {
            break;
        }
    }

    // Restore terminal
    restore_terminal(&mut terminal)?;
    Ok(())
}

fn should_pass_to_textarea(app: &App, event: &crossterm::event::Event) -> bool {
    use crossterm::event::{Event as CE, KeyCode, KeyModifiers};

    // Only pass to textarea when in chat input mode
    if app.focus != app::Focus::MainPanel
        || app.active_section != app::Section::Chat
        || app.chat.is_streaming
        || app.show_help
        || app.show_model_picker
    {
        return false;
    }

    if let CE::Key(key) = event {
        // These keys are handled by the keymap, not textarea
        match key.code {
            KeyCode::Enter if key.modifiers == KeyModifiers::NONE => false, // Send message
            KeyCode::Esc => false,
            KeyCode::Tab => false,
            KeyCode::Char('m') if key.modifiers.contains(KeyModifiers::CONTROL) => false,
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => false,
            KeyCode::Char('?') if key.modifiers.contains(KeyModifiers::CONTROL) => false,
            _ => true, // All other keys go to textarea
        }
    } else {
        false
    }
}

fn setup_terminal() -> Result<Terminal<CrosstermBackend<io::Stdout>>> {
    enable_raw_mode()?;
    execute!(stdout(), EnterAlternateScreen)?;

    // Install panic hook that restores terminal
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen);
        original_hook(panic_info);
    }));

    let backend = CrosstermBackend::new(stdout());
    Terminal::new(backend).map_err(Into::into)
}

fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}
