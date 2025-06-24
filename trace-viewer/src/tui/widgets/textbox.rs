use crate::{
    Component,
    tui::{ComponentStyle, ParentalFocusComponent, TuiComponent, TuiComponentBuilder},
};
use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    style::{Color, Style},
    widgets::Paragraph,
};
use std::str::FromStr;

/// A static block of text.
pub(crate) struct TextBox<D> {
    parent_has_focus: bool,
    data: D,
}

impl<D> TextBox<D>
where
    D: ToString + FromStr,
    <D as FromStr>::Err: std::fmt::Debug,
{
    /// Create's new textbox with the given content.
    ///
    /// The content can be any type which implements [ToString] and [FromStr]
    ///
    /// # Attribute
    /// - data: The content of the textbox.
    /// - name: if [Some] then display the given name on the textbox's border.
    pub(crate) fn new(data: D, name: Option<&'static str>) -> TuiComponent<Self> {
        let builder = TuiComponentBuilder::new(ComponentStyle::selectable()).is_in_block(true);

        if let Some(name) = name {
            builder.with_name(name)
        } else {
            builder
        }
        .build(Self {
            data,
            parent_has_focus: false,
        })
    }

    /// Set the textbox's content.
    pub(crate) fn set(&mut self, data: D) {
        self.data = data;
    }
}

impl<D> Component for TextBox<D>
where
    D: ToString,
{
    fn render(&self, frame: &mut Frame, area: Rect) {
        let style = Style::new().bg(Color::Black).fg(Color::Gray);

        let paragraph = Paragraph::new(self.data.to_string())
            .alignment(Alignment::Center)
            .style(style);
        frame.render_widget(paragraph, area);
    }
}

impl<D> ParentalFocusComponent for TextBox<D>
where
    D: ToString,
{
    fn propagate_parental_focus(&mut self, focus: bool) {
        self.parent_has_focus = focus;
    }
}
