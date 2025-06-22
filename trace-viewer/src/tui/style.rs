use ratatui::style::{Color, Style};

#[derive(Clone)]
pub(crate) struct ComponentStyle {
    pub(crate) full_focus: Style,
    pub(crate) only_parent_focus: Style,
    pub(crate) only_self_focus: Style,
    pub(crate) no_focus: Style,
}

impl ComponentStyle {
    pub(crate) fn default() -> Self {
        Self {
            full_focus: Style::new().fg(Color::LightGreen).bg(Color::Black),
            only_parent_focus: Style::new().fg(Color::Cyan).bg(Color::Black),
            only_self_focus: Style::new().fg(Color::Green).bg(Color::Black),
            no_focus: Style::new().fg(Color::DarkGray).bg(Color::Black),
        }
    }

    pub(crate) fn selectable() -> Self {
        Self {
            full_focus: Style::new().fg(Color::LightGreen).bg(Color::Black),
            only_parent_focus: Style::new().fg(Color::Cyan).bg(Color::Black),
            only_self_focus: Style::new().fg(Color::Green).bg(Color::Black),
            no_focus: Style::new().fg(Color::DarkGray).bg(Color::Black),
        }
    }
}
