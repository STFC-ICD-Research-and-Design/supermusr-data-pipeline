use crate::tui::{Component, TuiComponent, style::ComponentStyle};

pub(crate) struct TuiComponentBuilder {
    pub(crate) name: Option<&'static str>,
    pub(crate) style: ComponentStyle,
    pub(crate) is_in_block: bool,
}

impl TuiComponentBuilder {
    pub(crate) fn new(style: ComponentStyle) -> Self {
        Self {
            style,
            name: None,
            is_in_block: true,
        }
    }

    pub(crate) fn with_name(self, name: &'static str) -> Self {
        Self {
            style: self.style,
            name: Some(name),
            is_in_block: self.is_in_block,
        }
    }

    pub(crate) fn with_block(self, is_in_block: bool) -> Self {
        Self {
            style: self.style,
            name: self.name,
            is_in_block,
        }
    }

    pub(crate) fn build<C: Component>(self, comp: C) -> TuiComponent<C> {
        TuiComponent::new(comp, self)
    }
}
