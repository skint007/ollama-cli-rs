use ratatui::style::{Color, Modifier, Style};

pub struct Theme {
    pub border: Style,
    pub border_focused: Style,
    pub title: Style,
    pub sidebar_selected: Style,
    pub sidebar_unselected: Style,
    pub user_label: Style,
    pub assistant_label: Style,
    pub system_label: Style,
    pub status_bar: Style,
    pub status_connected: Style,
    pub status_disconnected: Style,
    pub error: Style,
    pub muted: Style,
    pub code_inline: Style,
    pub code_block_bg: Color,
    pub heading: Style,
    pub heading2: Style,
    pub bold: Style,
    pub italic: Style,
    pub link: Style,
    pub blockquote: Style,
    pub list_bullet: Style,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            border: Style::new().fg(Color::DarkGray),
            border_focused: Style::new().fg(Color::Cyan),
            title: Style::new().fg(Color::White).add_modifier(Modifier::BOLD),
            sidebar_selected: Style::new()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
            sidebar_unselected: Style::new().fg(Color::DarkGray),
            user_label: Style::new().fg(Color::Cyan).add_modifier(Modifier::BOLD),
            assistant_label: Style::new()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
            system_label: Style::new()
                .fg(Color::Yellow)
                .add_modifier(Modifier::DIM),
            status_bar: Style::new().fg(Color::White).bg(Color::DarkGray),
            status_connected: Style::new().fg(Color::Green),
            status_disconnected: Style::new().fg(Color::Red),
            error: Style::new().fg(Color::Red),
            muted: Style::new().fg(Color::DarkGray),
            code_inline: Style::new().fg(Color::Yellow),
            code_block_bg: Color::Rgb(30, 30, 30),
            heading: Style::new().fg(Color::Cyan).add_modifier(Modifier::BOLD),
            heading2: Style::new().fg(Color::Blue).add_modifier(Modifier::BOLD),
            bold: Style::new().add_modifier(Modifier::BOLD),
            italic: Style::new().add_modifier(Modifier::ITALIC),
            link: Style::new()
                .fg(Color::Blue)
                .add_modifier(Modifier::UNDERLINED),
            blockquote: Style::new().fg(Color::DarkGray),
            list_bullet: Style::new().fg(Color::DarkGray),
        }
    }
}

pub fn theme() -> Theme {
    Theme::default()
}
