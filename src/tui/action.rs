use super::app::Section;

#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    Quit,
    ToggleHelp,
    FocusSidebar,
    FocusMainPanel,
    NavigateUp,
    NavigateDown,
    SelectSection,
    JumpToSection(Section),

    // Chat
    SendMessage,
    ToggleModelPicker,
    ScrollUp,
    ScrollDown,
    ScrollPageUp,
    ScrollPageDown,
    ScrollToBottom,
    NewChat,
    FocusChatInput,

    // Models
    RefreshModels,
    ShowModelDetail,
    DeleteModel,

    // Running
    RefreshRunning,
    UnloadModel,

    // Model picker
    PickerConfirm,
    PickerCancel,

    None,
}
