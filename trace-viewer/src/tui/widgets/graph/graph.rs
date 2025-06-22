use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    symbols::Marker,
    widgets::{Chart, Dataset, GraphType, Scrollbar, ScrollbarOrientation, ScrollbarState},
    Frame,
};

use crate::{
    graphics::{Bound, Bounds},
    messages::{EventList, Trace},
    tui::{
        ComponentStyle, GraphProperties, ParentalFocusComponent, TuiComponent, TuiComponentBuilder,
    },
    Component,
};

/// Encapsulates and displays the [ratatui] graph of a given trace and eventlist.
pub(crate) struct Graph {
    /// Flag specifying whether an ancestor object has the focus or not.
    parent_has_focus: bool,
    /// The raw trace values of the graph.
    trace_data: Vec<(f64, f64)>,
    /// The raw event list of the graph, if present.
    event_data: Option<Vec<(f64, f64)>>,
    ///
    properties: Option<GraphProperties>,
    /// The current state of the horizontal scrollbar.
    hscroll_state: ScrollbarState,
    /// The current state of the vertical scrollbar.
    vscroll_state: ScrollbarState,
}

impl Graph {
    /// The width of the vertical scrollbar.
    const VSCROLL_BAR_WIDTH: u16 = 2;
    /// The height of the horizontal scrollbar.
    const HSCROLL_BAR_HEIGHT: u16 = 2;

    /// Create's new graph.
    pub(crate) fn new() -> TuiComponent<Self> {
        TuiComponentBuilder::new(ComponentStyle::selectable())
            .is_in_block(true)
            .build(Self {
                trace_data: Default::default(),
                event_data: None,
                parent_has_focus: false,
                properties: None,
                hscroll_state: ScrollbarState::default(),
                vscroll_state: ScrollbarState::default(),
            })
    }

    /// Sets the trace and eventlist data of the graph, and computes the [Self::properties].
    ///
    /// # Attributes
    /// - trace_data: the trace data to load.
    /// - event_data: the event list data to load, if available.
    pub(crate) fn set(&mut self, trace_data: &Trace, event_data: Option<&EventList>) {
        let trace_data: Vec<_> = (0_u32..).zip(trace_data.iter().copied()).collect();

        let event_data: Option<Vec<_>> =
            event_data.map(|events| events.iter().map(|e| (e.time, e.intensity)).collect());

        let time = trace_data.iter().map(|e| e.0 as u32);
        let time_bounds = Bound::from(1.0625, time.clone());

        let values = trace_data.iter().map(|e| e.1);
        let intensity_bounds = Bound::from(1.125, values.clone());

        let properties = GraphProperties::new(Bounds {
            time: time_bounds,
            intensity: intensity_bounds,
        });

        self.trace_data = trace_data
            .iter()
            .copied()
            .map(|(t, v)| (t as f64, v as f64))
            .collect();

        self.event_data = event_data.as_ref().map(|event_data| {
            event_data
                .iter()
                .copied()
                .map(|e| (e.0 as f64, e.1 as f64))
                .collect::<Vec<_>>()
        });

        self.properties = Some(properties);
        self.hscroll_state = ScrollbarState::new(100).viewport_content_length(100);
        self.vscroll_state = ScrollbarState::new(100).viewport_content_length(100);
    }

    /// Grants mutable access to the graph's properties object.
    pub(crate) fn get_properties_mut(&mut self) -> Option<&mut GraphProperties> {
        self.properties.as_mut()
    }

    /// Grants mutable access to the graph's properties object.
    pub(crate) fn get_properties(&self) -> Option<&GraphProperties> {
        self.properties.as_ref()
    }
}

impl Component for Graph {
    fn render(&self, frame: &mut Frame, area: Rect) {
        if let Some(properties) = &self.properties {
            // Infobar/Graph division
            let (_info, graph) = {
                let chunk = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Length(3), Constraint::Min(0)])
                    .split(area);
                (chunk[0], chunk[1])
            };

            //  Graph/Vscroll division
            let (graph, vscroll) = {
                let chunk = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([
                        Constraint::Min(0),
                        Constraint::Length(Self::VSCROLL_BAR_WIDTH),
                    ])
                    .split(graph);
                (chunk[0], chunk[1])
            };

            // Graph/Hscroll division
            let (graph, hscroll) = {
                let chunk1 = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Min(0),
                        Constraint::Length(Self::HSCROLL_BAR_HEIGHT),
                    ])
                    .split(graph);

                let chunk2 = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([
                        Constraint::Min(0),
                        Constraint::Length(Self::VSCROLL_BAR_WIDTH),
                    ])
                    .split(chunk1[1]);
                (chunk1[0], chunk2[0])
            };

            let horiz_scroll = Scrollbar::new(ScrollbarOrientation::HorizontalBottom);
            let vert_scroll = Scrollbar::new(ScrollbarOrientation::VerticalRight);
            frame.render_stateful_widget(horiz_scroll, hscroll, &mut self.hscroll_state.clone());
            frame.render_stateful_widget(vert_scroll, vscroll, &mut self.vscroll_state.clone());

            let trace_data = self
                .trace_data
                .iter()
                .copied()
                .filter(|(time, _)| {
                    properties.zoomed_bounds.time.min <= *time
                        && *time <= properties.zoomed_bounds.time.max
                })
                .collect::<Vec<_>>();

            let trace_dataset = Dataset::default()
                .name("Trace")
                .marker(Marker::Braille)
                .graph_type(GraphType::Line)
                .style(Style::new().fg(Color::Blue).bg(Color::Black))
                .data(trace_data.as_slice());

            let event_data = self.event_data.as_ref().map(|event_data| {
                event_data
                    .iter()
                    .copied()
                    .filter(|(time, _)| {
                        properties.zoomed_bounds.time.min <= *time
                            && *time <= properties.zoomed_bounds.time.max
                    })
                    .collect::<Vec<_>>()
            });

            let event_dataset = event_data.as_ref().map(|event_data| {
                Dataset::default()
                    .name("Events")
                    .marker(Marker::Dot)
                    .graph_type(GraphType::Scatter)
                    .style(Style::new().fg(Color::LightRed).bg(Color::Black))
                    .data(event_data.as_slice())
            });

            let datasets = if let Some(event_dataset) = event_dataset {
                vec![trace_dataset, event_dataset]
            } else {
                vec![trace_dataset]
            };

            let chart = Chart::new(datasets)
                .x_axis(properties.x_axis.clone())
                .y_axis(properties.y_axis.clone());

            frame.render_widget(chart, graph);
        }
    }
}

impl ParentalFocusComponent for Graph {
    fn propagate_parental_focus(&mut self, focus: bool) {
        self.parent_has_focus = focus;
    }
}
