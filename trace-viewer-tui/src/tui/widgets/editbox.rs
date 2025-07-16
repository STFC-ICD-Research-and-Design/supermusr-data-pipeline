use crate::{
    Component,
    tui::{
        ComponentStyle, FocusableComponent, InputComponent, ParentalFocusComponent, TuiComponent,
        TuiComponentBuilder,
    },
};
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    style::{Color, Style},
    widgets::Paragraph,
};
use std::str::FromStr;
use tui_input::Input;
use tui_input::backend::crossterm::EventHandler;

/// An editable terminal text box.
pub(crate) struct EditBox<D> {
    has_focus: bool,
    parent_has_focus: bool,
    data: D,
    input: Input,
    error: bool,
}

impl<D> EditBox<D>
where
    D: ToString + FromStr,
    <D as FromStr>::Err: std::fmt::Debug,
{
    /// Create's new edit box with the given vector of content.
    ///
    /// The content can be any type which implements [ToString] and [FromStr] (and for which [FromStr::Err] implements [Debug])
    ///
    /// # Parameters
    /// - data: The content of the textbox.
    /// - name: if [Some] then display the given name on the textbox's border.
    pub(crate) fn new(data: D, name: Option<&'static str>) -> TuiComponent<Self> {
        let input = Input::new(data.to_string());
        let builder = TuiComponentBuilder::new(ComponentStyle::selectable()).with_block(true);

        if let Some(name) = name {
            builder.with_name(name)
        } else {
            builder
        }
        .build(Self {
            input,
            data,
            has_focus: false,
            parent_has_focus: false,
            error: false,
        })
    }

    pub(crate) fn get(&self) -> &D {
        &self.data
    }
}

impl<D> Component for EditBox<D>
where
    D: ToString + FromStr,
    <D as FromStr>::Err: std::fmt::Debug,
{
    fn render(&self, frame: &mut Frame, area: Rect) {
        let style =
            Style::new()
                .bg(Color::Black)
                .fg(if self.error { Color::Red } else { Color::Gray });

        let paragraph = Paragraph::new(self.input.value())
            .alignment(Alignment::Center)
            .style(style);
        frame.render_widget(paragraph, area);
    }
}

impl<D> InputComponent for EditBox<D>
where
    D: ToString + FromStr,
    <D as FromStr>::Err: std::fmt::Debug,
{
    fn handle_key_event(&mut self, key: KeyEvent) {
        if self.has_focus {
            if key == KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE) {
                if self.input.visual_cursor() != 0 {
                    self.input.handle_event(&Event::Key(key)).expect("");
                }
            } else if let KeyEvent {
                code: KeyCode::Char(_),
                modifiers: _,
                kind: _,
                state: _,
            } = key
            {
                self.input.handle_event(&Event::Key(key)).expect("");
            }

            self.error = false;
            match self.input.value().parse() {
                Ok(value) => self.data = value,
                Err(_) => {
                    self.error = true;
                }
            }
        }
    }
}

impl<D> FocusableComponent for EditBox<D>
where
    D: ToString + FromStr,
    <D as FromStr>::Err: std::fmt::Debug,
{
    fn set_focus(&mut self, focus: bool) {
        self.has_focus = focus;
    }
}

impl<D> ParentalFocusComponent for EditBox<D>
where
    D: ToString + FromStr,
    <D as FromStr>::Err: std::fmt::Debug,
{
    fn propagate_parental_focus(&mut self, focus: bool) {
        self.parent_has_focus = focus;
    }
}
