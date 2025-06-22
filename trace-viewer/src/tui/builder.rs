use crate::tui::{style::ComponentStyle, Component, TuiComponent};

pub(crate) struct TuiComponentBuilder {
    pub(crate) name: Option<&'static str>,
    pub(crate) selected_name: Option<&'static str>,
    pub(crate) style: ComponentStyle,
    pub(crate) is_in_block: bool,
}

impl TuiComponentBuilder {
    pub(crate) fn new(style: ComponentStyle) -> Self {
        Self {
            style,
            name: None,
            selected_name: None,
            is_in_block: true,
        }
    }

    pub(crate) fn with_name(self, name: &'static str) -> Self {
        Self {
            style: self.style,
            name: Some(name),
            selected_name: self.selected_name,
            is_in_block: self.is_in_block,
        }
    }

    pub(crate) fn with_selected_name(self, selected_name: &'static str) -> Self {
        Self {
            style: self.style,
            name: self.name,
            selected_name: Some(selected_name),
            is_in_block: self.is_in_block,
        }
    }

    pub(crate) fn is_in_block(self, is_in_block: bool) -> Self {
        Self {
            style: self.style,
            name: self.name,
            selected_name: self.selected_name,
            is_in_block: is_in_block,
        }
    }

    pub(crate) fn build<C: Component>(self, comp: C) -> TuiComponent<C> {
        TuiComponent::new(comp, self)
    }
}
