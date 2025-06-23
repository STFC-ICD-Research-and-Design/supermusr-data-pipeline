use std::{ops::Deref, path::PathBuf};

use chrono::{NaiveDate, NaiveDateTime, NaiveTime, Utc};
use crossterm::event::KeyCode;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect, Spacing},
};
use strum::{EnumCount, EnumIter, IntoEnumIterator};
use supermusr_common::{Channel, DigitizerId};

use crate::{
    Component, Select, Timestamp,
    finder::{MessageFinder, SearchBy, SearchMode, SearchTarget},
    graphics::FileFormat,
    tui::{
        CSVVec, ComponentContainer, ComponentStyle, EditBox, FocusableComponent, InputComponent,
        ListBox, ParentalFocusComponent, TuiComponent, TuiComponentBuilder,
    },
};

#[derive(Default, Clone, EnumCount, EnumIter)]
pub(crate) enum Focus {
    #[default]
    SearchMode,
    Date,
    Time,
    Number,
    SearchBy,
    SearchCriteria,
    Format,
    SavePath,
    Width,
    Height,
}

pub(crate) struct Setup {
    focus: Focus,
    search_mode: TuiComponent<ListBox<SearchMode>>,
    date: TuiComponent<EditBox<NaiveDate>>,
    time: TuiComponent<EditBox<NaiveTime>>,
    number: TuiComponent<EditBox<usize>>,
    search_by: TuiComponent<ListBox<SearchBy>>,
    channel: TuiComponent<EditBox<CSVVec<Channel>>>,
    digitiser_id: TuiComponent<EditBox<CSVVec<DigitizerId>>>,
    save_path: TuiComponent<EditBox<String>>,
    format: TuiComponent<ListBox<FileFormat>>,
    width: TuiComponent<EditBox<u32>>,
    height: TuiComponent<EditBox<u32>>,
}

impl Setup {
    pub(crate) fn new(select: &Select) -> TuiComponent<Self> {
        let timestamp = select.timestamp.unwrap_or_else(Utc::now);
        let comp = Self {
            focus: Default::default(),
            search_mode: ListBox::new(
                &SearchMode::iter().collect::<Vec<_>>(),
                Some("Search Mode"),
                Some(0),
            )
                .with_tooltip("<Up>/<Down> to select search type."),
            date: EditBox::new(timestamp.date_naive(), Some("Date"))
                .with_tooltip("Date to search from (YYYY-MM-DD)."),
            time: EditBox::new(timestamp.time(), Some("Time"))
                .with_tooltip("Time to search from (hh:mm:ss.f)."),
            number: EditBox::new(1, Some("Number to Collect"))
                .with_tooltip("Max number of messages to collect."),
            search_by: ListBox::new(
                &SearchBy::iter().collect::<Vec<_>>(),
                Some("Search By"),
                Some(0),
            )
                .with_tooltip("<up>/<down> to select messages criteria."),
            channel: EditBox::new(select.channels.as_ref().cloned().unwrap_or_default(), Some("Channels"))
                .with_tooltip("Comma separatated list of channels, each message must include at least one of these."),
            digitiser_id: EditBox::new(select.digitiser_ids.as_ref().cloned().unwrap_or_default(), Some("Digitiser Ids"))
                .with_tooltip("Comma separatated list of digitiser ids, each message must matched one of these."),
            format: ListBox::new(
                &FileFormat::iter().collect::<Vec<_>>(),
                Some("Image Format"),
                Some(0),
            ).with_tooltip("Format to use whilst saving graph."),
            save_path: EditBox::new("Saves".to_owned(), Some("Save Path"))
                .with_tooltip("Directory in which graph images are saved, i.e ./<Save Path>/<Timestamp>/<Channel>.<format>"),
            width: EditBox::new(800, Some("Image Width"))
                .with_tooltip("Width of saved graph in pixels"),
            height: EditBox::new(600, Some("Image Height"))
                .with_tooltip("Height of saved graph in pixels"),
        };
        let mut setup = TuiComponentBuilder::new(ComponentStyle::default()).build(comp);
        setup.focused_mut().set_focus(true);
        setup
    }

    pub(crate) fn search<M: MessageFinder>(&self, message_finder: &mut M) {
        let timestamp = {
            let date = self.date.get();
            let time = self.time.get();
            Timestamp::from_naive_utc_and_offset(
                NaiveDateTime::new(date.clone(), time.clone()),
                Utc,
            )
        };
        let number = *self.number.get();
        if let Some((mode, by)) =
            Option::zip(self.search_mode.get_value(), self.search_by.get_value())
        {
            message_finder.init_search(SearchTarget {
                mode,
                by,
                timestamp,
                number,
                channels: self.channel.get().deref().to_owned(),
                digitiser_ids: self.digitiser_id.get().deref().to_owned(),
            });
        }
    }

    pub(crate) fn get_path(&self) -> PathBuf {
        PathBuf::from(self.save_path.get())
    }

    pub(crate) fn get_image_size(&self) -> (u32, u32) {
        (*self.width.get(), *self.height.get())
    }

    fn render_by_timestamp(&self, frame: &mut Frame, area: Rect) {
        // Date and Time/Search Params Division
        let (datetime, search_params) = {
            let chunk = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Ratio(1, 2); 2])
                .spacing(Spacing::Space(4))
                .split(area);
            (chunk[0], chunk[1])
        };

        // Date/Time Division
        let layout =
            Layout::new(Direction::Horizontal, [Constraint::Ratio(1, 3); 3]).split(datetime);
        self.date.render(frame, layout[0]);
        self.time.render(frame, layout[1]);
        self.number.render(frame, layout[2]);

        // Search Params Division
        self.render_search_params(frame, search_params);
    }

    fn render_only_number(&self, frame: &mut Frame, area: Rect) {
        // Date and Time/Search Params Division
        let (number, search_params) = {
            let chunk = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Ratio(1, 3), Constraint::Ratio(2, 3)])
                .spacing(Spacing::Space(4))
                .split(area);
            (chunk[0], chunk[1])
        };

        self.number.render(frame, number);
        // Date/Time Division
        self.render_search_params(frame, search_params);
    }

    fn render_search_params(&self, frame: &mut Frame, area: Rect) {
        // Search Params Division
        let (search_by, search_criteria) = {
            let chunk = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Ratio(1, 3), Constraint::Ratio(2, 3)])
                .split(area);
            (chunk[0], chunk[1])
        };
        self.search_by.render(frame, search_by);
        match self.search_by.get_value() {
            Some(SearchBy::ByChannels) => self.channel.render(frame, search_criteria),
            Some(SearchBy::ByDigitiserIds) => self.digitiser_id.render(frame, search_criteria),
            None => {}
        }
    }

    fn render_save_settings(&self, frame: &mut Frame, area: Rect) {
        let layout = Layout::new(
            Direction::Horizontal,
            [
                Constraint::Ratio(5, 16),
                Constraint::Ratio(5, 16),
                Constraint::Ratio(3, 16),
                Constraint::Ratio(3, 16),
            ],
        )
        .split(area);

        self.format.render(frame, layout[0]);
        self.save_path.render(frame, layout[1]);
        self.width.render(frame, layout[2]);
        self.height.render(frame, layout[3]);
    }
}

impl Component for Setup {
    fn render(&self, frame: &mut Frame, area: Rect) {
        // Search Mode Division
        let (search_mode, area) = {
            let chunk = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Length(24), Constraint::Min(0)])
                .spacing(Spacing::Space(6))
                .split(area);
            (chunk[0], chunk[1])
        };

        self.search_mode.render(frame, search_mode);

        // Settings/Save Division
        let (search_settings, save_settings) = {
            let chunk = Layout::new(
                Direction::Horizontal,
                [Constraint::Ratio(3, 5), Constraint::Ratio(2, 5)],
            )
            .spacing(Spacing::Space(4))
            .split(area);
            (chunk[0], chunk[1])
        };

        match self.search_mode.get_value() {
            Some(SearchMode::ByTimestamp) => self.render_by_timestamp(frame, search_settings),
            Some(SearchMode::FromEnd) => self.render_only_number(frame, search_settings),
            Some(SearchMode::Capture) => self.render_only_number(frame, search_settings),
            None => {}
        }

        // Save Settings Division
        self.render_save_settings(frame, save_settings);
    }
}

impl ComponentContainer for Setup {
    type Focus = Focus;

    fn get_focused_component(&self, focus: Focus) -> &dyn FocusableComponent {
        match focus {
            Focus::SearchMode => &self.search_mode,
            Focus::Date => &self.date,
            Focus::Time => &self.time,
            Focus::Number => &self.number,
            Focus::SearchBy => &self.search_by,
            Focus::SearchCriteria => match self.search_by.get_value() {
                Some(SearchBy::ByChannels) => &self.channel,
                Some(SearchBy::ByDigitiserIds) => &self.digitiser_id,
                None => &self.search_by,
            },
            Focus::SavePath => &self.save_path,
            Focus::Format => &self.format,
            Focus::Width => &self.width,
            Focus::Height => &self.height,
        }
    }
    fn get_focused_component_mut(&mut self, focus: Focus) -> &mut dyn FocusableComponent {
        match focus {
            Focus::SearchMode => &mut self.search_mode,
            Focus::Date => &mut self.date,
            Focus::Time => &mut self.time,
            Focus::Number => &mut self.number,
            Focus::SearchBy => &mut self.search_by,
            Focus::SearchCriteria => match self.search_by.get_value() {
                Some(SearchBy::ByChannels) => &mut self.channel,
                Some(SearchBy::ByDigitiserIds) => &mut self.digitiser_id,
                None => &mut self.search_by,
            },
            Focus::SavePath => &mut self.save_path,
            Focus::Format => &mut self.format,
            Focus::Width => &mut self.width,
            Focus::Height => &mut self.height,
        }
    }

    fn get_focus(&self) -> Self::Focus {
        self.focus.clone()
    }

    fn set_focus(&mut self, focus: Self::Focus) {
        self.focus = focus;
    }
}

impl InputComponent for Setup {
    fn handle_key_press(&mut self, key: crossterm::event::KeyEvent) {
        if key.code == KeyCode::Right {
            if let Some(SearchMode::ByTimestamp) = self.search_mode.get_value() {
                self.set_focus_index(self.focus.clone() as isize + 1);
            } else {
                if let Focus::SearchMode = self.focus.clone() {
                    self.set_focus_index(Focus::Number as isize);
                } else {
                    self.set_focus_index(self.focus.clone() as isize + 1)
                }
            }
        } else if key.code == KeyCode::Left {
            if let Some(SearchMode::ByTimestamp) = self.search_mode.get_value() {
                self.set_focus_index(self.focus.clone() as isize - 1)
            } else {
                if let Focus::Number = self.focus.clone() {
                    self.set_focus_index(Focus::SearchMode as isize);
                } else {
                    self.set_focus_index(self.focus.clone() as isize - 1)
                }
            }
        } else {
            self.focused_mut().handle_key_press(key);
        }
    }
}

impl FocusableComponent for Setup {
    fn set_focus(&mut self, focus: bool) {
        self.propagate_parental_focus(focus);
    }

    fn get_tooltip(&self) -> Option<&'static str> {
        self.focused().get_tooltip()
    }
}

impl ParentalFocusComponent for Setup {
    fn propagate_parental_focus(&mut self, focus: bool) {
        self.search_mode.propagate_parental_focus(focus);
        self.date.propagate_parental_focus(focus);
        self.time.propagate_parental_focus(focus);
        self.number.propagate_parental_focus(focus);
        self.search_by.propagate_parental_focus(focus);
        self.channel.propagate_parental_focus(focus);
        self.digitiser_id.propagate_parental_focus(focus);
        self.save_path.propagate_parental_focus(focus);
        self.format.propagate_parental_focus(focus);
    }
}
