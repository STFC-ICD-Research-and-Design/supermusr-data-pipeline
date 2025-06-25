
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, LineGauge},
};
use strum::{Display, EnumString};

use crate::{
    finder::{BrokerInfo, SearchResults, SearchStatus}, tui::{ComponentStyle, ParentalFocusComponent, TextBox, TuiComponent, TuiComponentBuilder}, Component
};

#[derive(Default, EnumString, Display)]
enum StatusMessage {
    #[default]
    #[strum(to_string = "Ready to Search. Press <Enter> to begin.")]
    Waiting,
    #[strum(to_string = "Searching for Traces.")]
    TraceSearchInProgress,
    #[strum(to_string = "Search for Trace Finished.")]
    TraceSearchFinished,
    #[strum(to_string = "Searching for Event Lists.")]
    EventListSearchInProgress,
    #[strum(to_string = "Search for Event Lists Finished.")]
    EventListSearchFinished,
    #[strum(to_string = "Search Complete. Found {num} traces, in {secs},{ms} ms. Press <Enter> to search again.")]
    SearchFinished{ num : usize, secs: i64, ms: i32 },
    #[strum(to_string = "{0}")]
    Text(String),
}

/// Displays status and progress of an ongoing search, and info relating to a completed search.
pub(crate) struct Statusbar {
    parent_has_focus: bool,
    info: TuiComponent<TextBox<String>>,
    status: TuiComponent<TextBox<StatusMessage>>,
    progress_steps: u32,
    total_steps: u32,
}

impl Statusbar {
    /// Create's new status bar.
    pub(crate) fn new() -> TuiComponent<Self> {
        TuiComponentBuilder::new(ComponentStyle::selectable())
            .with_block(true)
            .build(Self {
                parent_has_focus: false,
                info: TextBox::new(Default::default(), None),
                status: TextBox::new(Default::default(), Some("Status")),
                progress_steps: 0,
                total_steps: 1,
            })
    }

    /// Set the status bar to the given [SearchStatus].
    ///
    /// # Attribute
    /// - status: TODO.
    pub(crate) fn set_status(&mut self, status: SearchStatus) {
        match status {
            SearchStatus::Off => self.status.set(StatusMessage::Waiting),
            SearchStatus::TraceSearchInProgress(_prog) => {
                self.status.set(StatusMessage::TraceSearchInProgress);
                //self.progress_steps = prog + 1;
            }
            SearchStatus::TraceSearchFinished => {
                self.status.set(StatusMessage::TraceSearchFinished);
                //self.progress_steps = self.num_step_passes + 2;
            }
            SearchStatus::EventListSearchInProgress(_prog) => {
                self.status.set(StatusMessage::EventListSearchFinished);
                //self.progress_steps = prog + self.num_step_passes + 2;
            }
            SearchStatus::Successful { num, time } => {
                self.status.set(StatusMessage::SearchFinished { num, secs: time.num_seconds(), ms: time.subsec_millis() });
                //self.progress_steps = 2 * self.num_step_passes + 3;
            }
            SearchStatus::Text(text) => self.status.set(StatusMessage::Text(text)),
            SearchStatus::EventListSearchFinished => {}
        }
        //info!("{0}", self.status.get());
        //info!("{0}", self.progress_steps);
    }

    /// Set the info bar to the given [SearchResults]
    ///
    /// # Attribute
    /// - results: TODO.
    pub(crate) fn set_broker_info(&mut self, broker_info: BrokerInfo) {
        self.info.set(format!(
            "Broker has {} traces ({}-{})",
            broker_info.trace.offsets.1 - broker_info.trace.offsets.0,
            broker_info.trace.timestamps.0,
            broker_info.trace.timestamps.1
        ));
    }
}

impl Component for Statusbar {
    fn render(&self, frame: &mut Frame, area: Rect) {
        let (info, status, progress) = {
            let chunk = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Length(59),
                    Constraint::Min(32),
                    Constraint::Length(24),
                ])
                .split(area);
            (chunk[0], chunk[1], chunk[2])
        };

        let gauge = LineGauge::default()
            .block(Block::new().borders(Borders::ALL))
            .style(Style::new().fg(Color::LightGreen).bg(Color::Black))
            .ratio(self.progress_steps as f64 / self.total_steps as f64);

        self.info.render(frame, info);
        self.status.render(frame, status);
        frame.render_widget(gauge, progress);
    }
}

impl ParentalFocusComponent for Statusbar {
    fn propagate_parental_focus(&mut self, focus: bool) {
        self.parent_has_focus = focus;
    }
}
