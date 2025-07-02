use crate::tui::{
    BlockExt, Component, ComponentContainer, FocusableComponent, InputComponent,
    ParentalFocusComponent, builder::TuiComponentBuilder,
};
use crossterm::event::KeyEvent;
use ratatui::{
    Frame,
    layout::Rect,
    widgets::{Block, Borders},
};
use std::ops::{Deref, DerefMut};

/// A wrapper for all tui components which handles focus state
/// and ensures a border is rendered round it.
pub(crate) struct TuiComponent<C: Component + Sized> {
    /// Flag specifying whether this object has the focus or not.
    has_focus: bool,
    /// Flag specifying whether an ancestor object has the focus or not.
    parent_has_focus: bool,
    /// The underlying component.
    comp: C,
    /// The settings used to construct this component.
    config: TuiComponentBuilder,
    /// The component's tooltip
    tooltip: Option<&'static str>,
}

impl<C: Component> TuiComponent<C> {
    /// Create the wrapper object from the given component and builder settings.
    ///
    /// # Attriutes
    /// - comp: the underlying object.
    /// - config: the builder settings.
    pub(crate) fn new(comp: C, config: TuiComponentBuilder) -> Self {
        Self {
            has_focus: false,
            parent_has_focus: false,
            comp,
            config,
            tooltip: None,
        }
    }

    pub(crate) fn with_tooltip(self, tooltip: &'static str) -> Self {
        Self {
            has_focus: self.has_focus,
            parent_has_focus: self.parent_has_focus,
            comp: self.comp,
            config: self.config,
            tooltip: Some(tooltip),
        }
    }

    pub(crate) fn has_focus(&self) -> bool {
        self.has_focus
    }

    pub(crate) fn parent_has_focus(&self) -> bool {
        self.parent_has_focus
    }

    pub(crate) fn get_builder(&self) -> &TuiComponentBuilder {
        &self.config
    }
}

/// This implementation coerces any reference to a reference of the underlying component.
/// This allows the `&self` methods of the underlying component to be called on this object.
impl<D> Deref for TuiComponent<D>
where
    D: Component,
{
    type Target = D;

    fn deref(&self) -> &Self::Target {
        &self.comp
    }
}

/// This implementation coerces any mutable reference to a mutable reference of the underlying component.
/// This allows the `&mut self` methods of the underlying component to be called on this object.
impl<D> DerefMut for TuiComponent<D>
where
    D: Component,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.comp
    }
}

impl<C: ComponentContainer> ComponentContainer for TuiComponent<C> {
    type Focus = C::Focus;

    fn get_focused_component(&self, focus: Self::Focus) -> &dyn FocusableComponent {
        self.comp.get_focused_component(focus)
    }

    fn get_focused_component_mut(&mut self, focus: Self::Focus) -> &mut dyn FocusableComponent {
        self.comp.get_focused_component_mut(focus)
    }

    fn get_focus(&self) -> Self::Focus {
        self.comp.get_focus()
    }

    fn set_focus(&mut self, focus: Self::Focus) {
        self.comp.set_focus(focus);
    }
}

impl<C: Component> Component for TuiComponent<C> {
    fn render(&self, frame: &mut Frame, area: Rect) {
        if self.config.is_in_block {
            let block = Block::new()
                .borders(Borders::ALL)
                .set_title(self)
                .set_border(self);

            frame.render_widget(block.clone(), area);
            self.comp.render(frame, block.inner(area));
        } else {
            self.comp.render(frame, area);
        };
    }
}

impl<C: InputComponent> InputComponent for TuiComponent<C> {
    fn handle_key_event(&mut self, key: KeyEvent) {
        self.comp.handle_key_event(key)
    }
}

impl<C: FocusableComponent> FocusableComponent for TuiComponent<C> {
    fn set_focus(&mut self, focus: bool) {
        self.has_focus = focus;
        self.comp.set_focus(focus);
    }

    fn get_tooltip(&self) -> Option<&'static str> {
        self.tooltip
    }
}

impl<C: ParentalFocusComponent> ParentalFocusComponent for TuiComponent<C> {
    fn propagate_parental_focus(&mut self, focus: bool) {
        self.parent_has_focus = focus;
        self.comp.propagate_parental_focus(focus);
    }
}
