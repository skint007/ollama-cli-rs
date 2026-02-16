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
    FocusChatMessages,

    // Models
    RefreshModels,
    ShowModelDetail,
    DeleteModel,
    PullModel,
    CopyModel,

    // Running
    RefreshRunning,
    UnloadModel,

    // Confirmation dialog
    ConfirmYes,
    ConfirmNo,

    // Model picker
    PickerConfirm,
    PickerCancel,

    // Close overlay
    CloseOverlay,

    None,
}
