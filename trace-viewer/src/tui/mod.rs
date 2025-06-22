mod builder;
mod style;
mod tui_component;
mod widgets;

use std::{ops::Deref, str::FromStr};

use crossterm::event::KeyEvent;
use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    widgets::{Block, BorderType},
};
use strum::{EnumCount, IntoEnumIterator};

pub(crate) use builder::TuiComponentBuilder;

pub(crate) use style::ComponentStyle;
pub(crate) use tui_component::TuiComponent;
pub(crate) use widgets::{Channels, EditBox, Graph, GraphProperties, ListBox, Statusbar, TextBox};

/// Provides method to render any component in a [Frame]
pub(crate) trait Component {
    /// Uses [Frame] to render the component in `area`.
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

    fn focused_component_mut(&mut self) -> &mut dyn FocusableComponent {
        let focus = self.get_focus();
        self.get_focused_component_mut(focus)
    }

    fn focused_component(&self) -> &dyn FocusableComponent {
        let focus = self.get_focus();
        self.get_focused_component(focus)
    }

    /// Sets the focus to the given index (mod the number of children).
    fn set_focus_index(&mut self, index: isize) {
        self.focused_component_mut().set_focus(false);
        self.set_focus(
            Self::Focus::iter()
                .cycle()
                .skip((Self::Focus::COUNT as isize + index) as usize % Self::Focus::COUNT)
                .next()
                .expect(""),
        );
        self.focused_component_mut().set_focus(true);
    }
}

/// Provides method to handle user key events.
///
/// This method does not return any value, so all results of user input should
/// be stored as internal state.
pub(crate) trait InputComponent: Component {
    ///
    fn handle_key_press(&mut self, key: KeyEvent);
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

/// Provides handling for informing a component when an ancestor has been given, or lost the focus.
pub(crate) trait ParentalFocusComponent: Component {
    ///
    fn propagate_parental_focus(&mut self, focus: bool);
}

#[derive(Default, Clone, Debug)]
pub(crate) struct CSVVec<T>(Vec<T>);

impl<T> CSVVec<T> {
    pub(crate) fn new(data: Vec<T>) -> Self {
        Self(data)
    }
}

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

impl<T> ToString for CSVVec<T>
where
    T: ToString,
{
    fn to_string(&self) -> String {
        self.0
            .iter()
            .map(T::to_string)
            .collect::<Vec<_>>()
            .join(",")
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
        let name = if comp.has_focus() {
            comp.get_builder().selected_name.or(comp.get_builder().name)
        } else {
            comp.get_builder().name
        };
        if let Some(name) = name {
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
        } else {
            if comp.parent_has_focus() {
                self.border_style(comp.get_builder().style.only_parent_focus)
            } else {
                self.border_style(comp.get_builder().style.no_focus)
            }
        }
    }
}
