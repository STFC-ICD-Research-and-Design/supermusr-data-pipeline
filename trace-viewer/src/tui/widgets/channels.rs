use crate::{
    Component,
    tui::{
        ComponentStyle, FocusableComponent, InputComponent, ParentalFocusComponent, TuiComponent,
        TuiComponentBuilder,
    },
};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::Tabs,
};
use supermusr_common::Channel;

#[derive(Clone)]
pub(crate) struct Channels {
    has_focus: bool,
    parent_has_focus: bool,
    channels: Vec<Channel>,
    channel_index: usize,
}

impl Channels {
    /// Create's new channels box.
    pub(crate) fn new() -> TuiComponent<Self> {
        TuiComponentBuilder::new(ComponentStyle::selectable())
            .with_block(true)
            .with_name("Channels")
            .build(Self {
                channels: Default::default(),
                has_focus: false,
                parent_has_focus: false,
                channel_index: 0,
            })
    }

    pub(crate) fn set(&mut self, channels: Vec<Channel>) {
        self.channels = channels;
        if self.channel_index >= self.channels.len() {
            self.channel_index = 0;
        }
    }

    pub(crate) fn get(&self) -> Option<Channel> {
        if self.channels.is_empty() {
            None
        } else {
            Some(*self.channels.get(self.channel_index)?)
        }
    }
}

impl Component for Channels {
    fn render(&self, frame: &mut Frame, area: Rect) {
        if self.channels.is_empty() {
            return;
        }

        let style = Style::new().bg(Color::Rgb(0, 64, 0)).fg(Color::Gray);
        let select_style = Style::new()
            .bg(Color::Green)
            .fg(Color::Black)
            .add_modifier(Modifier::BOLD);

        let tabs = Tabs::new(self.channels.iter().map(|c| format!(" {c} ")))
            .style(style)
            .highlight_style(select_style)
            //.divider(symbols::line::THICK_VERTICAL)
            .select(self.channel_index);

        frame.render_widget(tabs, area);
    }
}

impl InputComponent for Channels {
    fn handle_key_event(&mut self, key: KeyEvent) {
        if self.channels.is_empty() {
            return;
        }
        if self.has_focus {
            if key.code == KeyCode::Left {
                self.channel_index =
                    (self.channels.len() + self.channel_index - 1) % self.channels.len();
            } else if key.code == KeyCode::Right {
                self.channel_index = (self.channel_index + 1) % self.channels.len();
            }
        }
    }
}

impl ParentalFocusComponent for Channels {
    fn propagate_parental_focus(&mut self, focus: bool) {
        self.parent_has_focus = focus;
    }
}

impl FocusableComponent for Channels {
    fn set_focus(&mut self, focus: bool) {
        self.has_focus = focus;
    }
}
