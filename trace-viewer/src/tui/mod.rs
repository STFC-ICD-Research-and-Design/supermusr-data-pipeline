//! Provides traits and structs to implement complex [ratatui] components.
mod builder;
mod style;
mod tui_component;
mod widgets;

use crossterm::event::KeyEvent;
use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    widgets::{Block, BorderType},
};
use std::{fmt::Display, ops::Deref, str::FromStr};
use strum::{EnumCount, IntoEnumIterator};

pub(crate) use builder::TuiComponentBuilder;
pub(crate) use style::ComponentStyle;
pub(crate) use tui_component::TuiComponent;
pub(crate) use widgets::{Channels, EditBox, Graph, GraphProperties, ListBox, Statusbar, TextBox};

/// Provides method to render any component in a [Frame]
pub(crate) trait Component {
    /// Uses [Frame] to render the component in `area`.
    ///
    /// # Parameters
    /// - frame: renders the component to the terminal.
    /// - area: the rectangle the component is rendered to.
    fn render(&self, frame: &mut Frame, area: Rect);
}

/// Provides methods for components which contain other components, and have a `Focus` function.
pub(crate) trait ComponentContainer: Component {
    /// The `enum` type which defines the child coponents.
    type Focus: IntoEnumIterator + EnumCount;

    /// Maps each variant of [Self::Focus] to a type implementing [FocusableComponent].
    fn get_focused_component(&self, focus: Self::Focus) -> &dyn FocusableComponent;

    /// Maps each variant of [Self::Focus] to a type implementing [FocusableComponent].
    fn get_focused_component_mut(&mut self, focus: Self::Focus) -> &mut dyn FocusableComponent;

    /// Gets the current focus.
    fn get_focus(&self) -> Self::Focus;

    /// Sets the current focus.
    fn set_focus(&mut self, focus: Self::Focus);

    /// Returns a mutable reference to the currently focused component.
    fn focused_mut(&mut self) -> &mut dyn FocusableComponent {
        let focus = self.get_focus();
        self.get_focused_component_mut(focus)
    }

    /// Returns an immutable reference to the currently focused component.
    fn focused(&self) -> &dyn FocusableComponent {
        let focus = self.get_focus();
        self.get_focused_component(focus)
    }

    /// Sets the focus to the given index (mod the number of children).
    fn set_focus_index(&mut self, index: isize) {
        self.focused_mut().set_focus(false);
        self.set_focus(
            Self::Focus::iter()
                .nth((Self::Focus::COUNT as isize + index) as usize % Self::Focus::COUNT)
                .expect("nth should return Some(), this should never fail."),
        );
        self.focused_mut().set_focus(true);
    }
}

/// Provides method to handle user key events.
///
/// This method does not return any value, so all results of user input should
/// be stored as internal state.
pub(crate) trait InputComponent: Component {
    /// pass the key event to the component to handle.
    fn handle_key_event(&mut self, key: KeyEvent);
}

/// Provides handling for components which can be given the focus by the user
/// or its parent component.
pub(crate) trait FocusableComponent: InputComponent {
    /// Set the focus on or off for this component.
    ///
    /// Only one child component should ever have the focus at one time.
    /// This is not checked, so the parent is responsible for ensuring this behaviour.
    fn set_focus(&mut self, focus: bool);

    fn get_tooltip(&self) -> Option<&'static str> {
        None
    }
}

/// Provides handling for components whose style or function may be depend on whether an ancestor is focused.
pub(crate) trait ParentalFocusComponent: Component {
    /// Inform the component that an ancestor has the focus.
    fn propagate_parental_focus(&mut self, focus: bool);
}

/// A wrapper for a [Vec] which may be converted to and from a string of comma separated values.
#[derive(Default, Clone, Debug)]
pub(crate) struct CSVVec<T>(Vec<T>);

impl<T> FromStr for CSVVec<T>
where
    T: FromStr,
{
    type Err = T::Err;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.split(",")
            .map(T::from_str)
            .collect::<Result<_, _>>()
            .map(Self)
    }
}

impl<T> Display for CSVVec<T>
where
    T: Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            self.0
                .iter()
                .map(T::to_string)
                .collect::<Vec<_>>()
                .join(",")
        )
    }
}

impl<T> Deref for CSVVec<T> {
    type Target = Vec<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Handles complex title and border handling which depends on the state of a component.
pub(crate) trait BlockExt {
    /// Sets the block title to the correct content and style according to the state of `comp`.
    fn set_title<C: Component>(self, comp: &TuiComponent<C>) -> Self;

    /// Sets the block border to the correct style according to the state of `comp`.
    fn set_border<C: Component>(self, comp: &TuiComponent<C>) -> Self;
}

impl BlockExt for Block<'_> {
    fn set_title<C: Component>(self, comp: &TuiComponent<C>) -> Self {
        if let Some(name) = comp.get_builder().name {
            self.title_top(name).title_alignment(Alignment::Center)
        } else {
            self
        }
    }

    fn set_border<C: Component>(self, comp: &TuiComponent<C>) -> Self {
        if comp.has_focus() {
            if comp.parent_has_focus() {
                self.border_style(comp.get_builder().style.full_focus)
                    .border_type(BorderType::Rounded)
            } else {
                self.border_style(comp.get_builder().style.only_self_focus)
                    .border_type(BorderType::Rounded)
            }
        } else if comp.parent_has_focus() {
            self.border_style(comp.get_builder().style.only_parent_focus)
        } else {
            self.border_style(comp.get_builder().style.no_focus)
        }
    }
}
