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
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    symbols,
    widgets::{List, ListItem, ListState, Scrollbar, ScrollbarOrientation, ScrollbarState},
};
use std::str::FromStr;
use strum::IntoEnumIterator;

/// A vertical list of text options, selectable by the user.
pub(crate) struct ListBox<D> {
    has_state_changed: bool,
    has_focus: bool,
    parent_has_focus: bool,
    data: Vec<D>,
    state: ListState,
}

impl<D> ListBox<D>
where
    D: Clone + ToString + FromStr,
    <D as FromStr>::Err: std::fmt::Debug,
{
    /// Create's new list box with the given vector of content.
    ///
    /// The content can be any type which implements [ToString] and [FromStr] (and for which [FromStr::Err] implements [Debug])
    ///
    /// # Parameters
    /// - data: The content of the textbox.
    /// - name: if [Some] then display the given name on the textbox's border.
    /// - index: if [Some] then set the list box index to the given index.
    pub(crate) fn new(
        data: &[D],
        name: Option<&'static str>,
        index: Option<usize>,
    ) -> TuiComponent<Self> {
        let builder = TuiComponentBuilder::new(ComponentStyle::selectable()).with_block(true);

        if let Some(name) = name {
            builder.with_name(name)
        } else {
            builder
        }
        .build(Self {
            data: data.to_vec(),
            has_focus: false,
            parent_has_focus: false,
            state: ListState::default().with_selected(index),
            has_state_changed: true,
        })
    }

    pub(crate) fn set(&mut self, data: Vec<D>) {
        self.data = data;
        self.state = ListState::default()
    }

    pub(crate) fn get_value(&self) -> Option<D>
    where
        D: IntoEnumIterator + Copy,
    {
        self.state
            .selected()
            .and_then(|i| self.data.get(i).copied())
    }

    pub(crate) fn get_index(&self) -> Option<usize> {
        if self.data.is_empty() {
            None
        } else {
            self.state.selected()
        }
    }

    pub(crate) fn pop_state_change(&mut self) -> bool {
        let old_state_change = self.has_state_changed;
        if self.has_state_changed {
            self.has_state_changed = false;
        }
        old_state_change
    }
}

impl<D> Component for ListBox<D>
where
    D: Clone + ToString + FromStr,
    <D as FromStr>::Err: std::fmt::Debug,
{
    fn render(&self, frame: &mut Frame, area: Rect) {
        let style = Style::new().bg(Color::Black).fg(Color::Gray);
        let select_style = Style::new().bg(Color::Green).fg(Color::Black);

        let (list_area, scrollbar_area) = {
            let chunk = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Min(0), Constraint::Max(2)])
                .split(area);
            (chunk[0], chunk[1])
        };

        let list = List::new(
            self.data
                .iter()
                .map(ToString::to_string)
                .map(ListItem::new)
                .collect::<Vec<_>>(),
        )
        .style(style)
        .highlight_symbol(symbols::bar::THREE_EIGHTHS)
        .highlight_style(select_style);

        frame.render_stateful_widget(list, list_area, &mut self.state.clone());

        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight);
        let mut scrollbar_state = ScrollbarState::default().content_length(18);

        frame.render_stateful_widget(scrollbar, scrollbar_area, &mut scrollbar_state);
    }
}

impl<D> InputComponent for ListBox<D>
where
    D: Clone + ToString + FromStr,
    <D as FromStr>::Err: std::fmt::Debug,
{
    fn handle_key_event(&mut self, key: KeyEvent) {
        if self.data.is_empty() {
            return;
        }
        if self.has_focus {
            if key.code == KeyCode::Up {
                if let Some(selection) = self.state.selected() {
                    self.state
                        .select(Some((self.data.len() + selection - 1) % self.data.len()));
                } else {
                    self.state.select(Some(0));
                }
                self.has_state_changed = true;
            } else if key.code == KeyCode::Down {
                if let Some(selection) = self.state.selected() {
                    self.state.select(Some((selection + 1) % self.data.len()));
                } else {
                    self.state.select(Some(0));
                }
                self.has_state_changed = true;
            }
        }
    }
}

impl<D> ParentalFocusComponent for ListBox<D>
where
    D: Clone + ToString + FromStr,
    <D as FromStr>::Err: std::fmt::Debug,
{
    fn propagate_parental_focus(&mut self, focus: bool) {
        self.parent_has_focus = focus;
    }
}

impl<D> FocusableComponent for ListBox<D>
where
    D: Clone + ToString + FromStr,
    <D as FromStr>::Err: std::fmt::Debug,
{
    fn set_focus(&mut self, focus: bool) {
        self.has_focus = focus;
    }
}
