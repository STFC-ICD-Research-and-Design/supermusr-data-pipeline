use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    Frame,
};

use crate::{
    messages::{EventList, Trace},
    tui::{
        ComponentStyle, FocusableComponent, Graph, GraphProperties, InputComponent,
        ParentalFocusComponent, TextBox, TuiComponent, TuiComponentBuilder,
    },
    Component,
};

pub(crate) struct Display {
    info: TuiComponent<TextBox<String>>,
    graph: TuiComponent<Graph>,
}

impl Display {
    pub(crate) fn new() -> TuiComponent<Self> {
        TuiComponentBuilder::new(ComponentStyle::selectable()).build(Self {
            info: TextBox::new(Default::default(), None),
            graph: Graph::new(),
        })
    }

    pub(crate) fn select(&mut self, trace_data: &Trace, event_data: Option<&EventList>) {
        self.graph.set(trace_data, event_data);
        if let Some(properties) = self.graph.get_properties() {
            self.info.set(format!("{}", properties.get_info()));
        }
    }

    /*pub(crate) fn set_info(&mut self, info: &str) {
        self.info.set(info.to_string())
    }*/
}

impl ParentalFocusComponent for Display {
    fn propagate_parental_focus(&mut self, focus: bool) {
        self.graph.propagate_parental_focus(focus);
    }
}

impl Component for Display {
    fn render(&self, frame: &mut Frame, area: Rect) {
        let (info, results) = {
            let chunk = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(4), Constraint::Min(16)])
                .split(area);
            (chunk[0], chunk[1])
        };

        self.info.render(frame, info);
        self.graph.render(frame, results);
    }
}

impl InputComponent for Display {
    fn handle_key_press(&mut self, key: KeyEvent) {
        if let Some(properties) = self.graph.get_properties_mut() {
            if key.code == KeyCode::Char('+') {
                properties.zoom_in();
            } else if key.code == KeyCode::Char('-') {
                properties.zoom_out();
            } else if key.code == KeyCode::Up {
                properties.move_viewport(0.0, 1.0);
            } else if key.code == KeyCode::Down {
                properties.move_viewport(0.0, -1.0);
            } else if key.code == KeyCode::Left {
                properties.move_viewport(-1.0, 0.0);
            } else if key.code == KeyCode::Right {
                properties.move_viewport(1.0, 0.0);
            }
            self.info.set(format!("{}", properties.get_info()))
        }
    }
}

impl FocusableComponent for Display {
    fn set_focus(&mut self, focus: bool) {
        self.propagate_parental_focus(focus);
    }
}
