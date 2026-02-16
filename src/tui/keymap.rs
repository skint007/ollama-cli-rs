use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use super::action::Action;
use super::app::Section;

#[derive(Debug, Clone, PartialEq)]
pub enum InputContext {
    Global,
    Sidebar,
    ChatInput,
    ChatMessages,
    ModelsTable,
    RunningTable,
    ConfigTable,
}

pub fn map_key_event(key: KeyEvent, context: &InputContext) -> Action {
    // Global bindings that work everywhere
    match key.code {
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            return Action::Quit;
        }
        _ => {}
    }

    match context {
        InputContext::Sidebar => map_sidebar_key(key),
        InputContext::ChatInput => map_chat_input_key(key),
        InputContext::ChatMessages => map_chat_messages_key(key),
        InputContext::ModelsTable => map_models_key(key),
        InputContext::RunningTable => map_running_key(key),
        InputContext::ConfigTable => map_config_key(key),
        InputContext::Global => map_global_key(key),
    }
}

fn map_global_key(key: KeyEvent) -> Action {
    match key.code {
        KeyCode::Char('q') => Action::Quit,
        KeyCode::Char('?') => Action::ToggleHelp,
        KeyCode::Tab | KeyCode::BackTab => Action::FocusSidebar,
        KeyCode::Char('1') => Action::JumpToSection(Section::Chat),
        KeyCode::Char('2') => Action::JumpToSection(Section::Models),
        KeyCode::Char('3') => Action::JumpToSection(Section::Running),
        KeyCode::Char('4') => Action::JumpToSection(Section::Library),
        KeyCode::Char('5') => Action::JumpToSection(Section::Benchmarks),
        KeyCode::Char('6') => Action::JumpToSection(Section::Config),
        _ => Action::None,
    }
}

fn map_sidebar_key(key: KeyEvent) -> Action {
    match key.code {
        KeyCode::Char('q') => Action::Quit,
        KeyCode::Char('?') => Action::ToggleHelp,
        KeyCode::Char('j') | KeyCode::Down => Action::NavigateDown,
        KeyCode::Char('k') | KeyCode::Up => Action::NavigateUp,
        KeyCode::Enter | KeyCode::Char('l') | KeyCode::Right => Action::SelectSection,
        KeyCode::Char('1') => Action::JumpToSection(Section::Chat),
        KeyCode::Char('2') => Action::JumpToSection(Section::Models),
        KeyCode::Char('3') => Action::JumpToSection(Section::Running),
        KeyCode::Char('4') => Action::JumpToSection(Section::Library),
        KeyCode::Char('5') => Action::JumpToSection(Section::Benchmarks),
        KeyCode::Char('6') => Action::JumpToSection(Section::Config),
        _ => Action::None,
    }
}

fn map_chat_input_key(key: KeyEvent) -> Action {
    match key {
        KeyEvent {
            code: KeyCode::Enter,
            modifiers: KeyModifiers::NONE,
            ..
        } => Action::SendMessage,
        KeyEvent {
            code: KeyCode::Char('m'),
            modifiers,
            ..
        } if modifiers.contains(KeyModifiers::ALT) => Action::ToggleModelPicker,
        KeyEvent {
            code: KeyCode::Esc, ..
        } => Action::FocusChatMessages,
        KeyEvent {
            code: KeyCode::Char('?'),
            modifiers,
            ..
        } if modifiers.contains(KeyModifiers::CONTROL) => Action::ToggleHelp,
        KeyEvent {
            code: KeyCode::Tab, ..
        } => Action::FocusSidebar,
        _ => Action::None, // Will be passed to textarea
    }
}

fn map_chat_messages_key(key: KeyEvent) -> Action {
    match key.code {
        KeyCode::Char('q') => Action::Quit,
        KeyCode::Char('?') => Action::ToggleHelp,
        KeyCode::Char('j') | KeyCode::Down => Action::ScrollDown,
        KeyCode::Char('k') | KeyCode::Up => Action::ScrollUp,
        KeyCode::Char('d') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            Action::ScrollPageDown
        }
        KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            Action::ScrollPageUp
        }
        KeyCode::Char('G') => Action::ScrollToBottom,
        KeyCode::Char('i') | KeyCode::Enter => Action::FocusChatInput,
        KeyCode::Char('n') => Action::NewChat,
        KeyCode::Tab | KeyCode::BackTab => Action::FocusSidebar,
        KeyCode::Esc => Action::FocusSidebar,
        KeyCode::Char('m') if key.modifiers.contains(KeyModifiers::ALT) => {
            Action::ToggleModelPicker
        }
        _ => Action::None,
    }
}

fn map_models_key(key: KeyEvent) -> Action {
    match key.code {
        KeyCode::Char('q') => Action::Quit,
        KeyCode::Char('?') => Action::ToggleHelp,
        KeyCode::Char('j') | KeyCode::Down => Action::NavigateDown,
        KeyCode::Char('k') | KeyCode::Up => Action::NavigateUp,
        KeyCode::Enter => Action::ShowModelDetail,
        KeyCode::Char('r') => Action::RefreshModels,
        KeyCode::Char('d') => Action::DeleteModel,
        KeyCode::Char('p') => Action::PullModel,
        KeyCode::Char('c') => Action::CopyModel,
        KeyCode::Tab | KeyCode::BackTab => Action::FocusSidebar,
        KeyCode::Esc => Action::FocusSidebar,
        _ => Action::None,
    }
}

fn map_running_key(key: KeyEvent) -> Action {
    match key.code {
        KeyCode::Char('q') => Action::Quit,
        KeyCode::Char('?') => Action::ToggleHelp,
        KeyCode::Char('j') | KeyCode::Down => Action::NavigateDown,
        KeyCode::Char('k') | KeyCode::Up => Action::NavigateUp,
        KeyCode::Char('u') => Action::UnloadModel,
        KeyCode::Char('r') => Action::RefreshRunning,
        KeyCode::Tab | KeyCode::BackTab => Action::FocusSidebar,
        KeyCode::Esc => Action::FocusSidebar,
        _ => Action::None,
    }
}

fn map_config_key(key: KeyEvent) -> Action {
    match key.code {
        KeyCode::Char('q') => Action::Quit,
        KeyCode::Char('?') => Action::ToggleHelp,
        KeyCode::Char('j') | KeyCode::Down => Action::NavigateDown,
        KeyCode::Char('k') | KeyCode::Up => Action::NavigateUp,
        KeyCode::Enter => Action::ConfigSwitchProfile,
        KeyCode::Char('a') => Action::ConfigAddProfile,
        KeyCode::Char('d') => Action::ConfigRemoveProfile,
        KeyCode::Char('t') => Action::ConfigTestConnection,
        KeyCode::Tab | KeyCode::BackTab => Action::FocusSidebar,
        KeyCode::Esc => Action::FocusSidebar,
        _ => Action::None,
    }
}
