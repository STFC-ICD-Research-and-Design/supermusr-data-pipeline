use crossterm::event::KeyEvent;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
};
use std::cmp::Ordering;
use supermusr_common::Channel;

use crate::{
    Component,
    messages::{Cache, DigitiserMetadata, DigitiserTrace},
    tui::{
        Channels, ComponentStyle, FocusableComponent, InputComponent, ListBox,
        ParentalFocusComponent, TuiComponent, TuiComponentBuilder,
    },
};

pub(crate) struct Results {
    list: TuiComponent<ListBox<String>>,
    channels: TuiComponent<Channels>,
}

impl Results {
    pub(crate) fn new() -> TuiComponent<Self> {
        TuiComponentBuilder::new(ComponentStyle::selectable()).build(Self {
            list: ListBox::new(&[], Some("Traces"), None),
            channels: Channels::new(),
        })
    }

    pub(crate) fn new_cache(&mut self, cache: &Cache) {
        let mut list = cache.iter().collect::<Vec<_>>();

        list.sort_by(|x, y| {
            let datetime = x.0.timestamp.cmp(&y.0.timestamp);
            match datetime {
                Ordering::Equal => x.0.id.cmp(&y.0.id),
                other => other,
            }
        });

        let list = list
            .into_iter()
            .map(|(metadata, trace)| {
                format!(
                    "[{}]\nid: {}, num channels {}, num_bins: {}",
                    metadata.timestamp,
                    metadata.id,
                    trace.traces.len(),
                    trace.traces
                        .values()
                        .map(Vec::len)
                        .max()
                        .unwrap_or_default()
                )
            })
            .collect();

        self.list.set(list);
    }

    pub(crate) fn get_selected_trace<'a>(
        &mut self,
        cache: &'a Cache,
    ) -> Option<(&'a DigitiserMetadata, &'a DigitiserTrace, Channel)> {
        self.list
            .get_index()
            .and_then(|i| cache.iter().nth(i))
            .and_then(|(m, t)| self.channels.get().map(|c| (m, t, c)))
    }

    /// If the state of [Self::list] has changed, then rebuild the channel list.
    pub(crate) fn update(&mut self, cache: &Cache) {
        if self.list.pop_state_change() {
            let mut channels = self
                .list
                .get_index()
                .and_then(|i| cache.iter().nth(i))
                .map(|(_, trace)| trace.traces.keys().copied().collect::<Vec<_>>())
                .unwrap_or_default();
            channels.sort();
            self.channels.set(channels);
        }
    }
}

impl Component for Results {
    fn render(&self, frame: &mut Frame, area: Rect) {
        let (list, channels) = {
            let chunk = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(4), Constraint::Length(3)])
                .split(area);
            (chunk[0], chunk[1])
        };

        self.list.render(frame, list);
        self.channels.render(frame, channels);
        //}
    }
}

impl InputComponent for Results {
    fn handle_key_event(&mut self, key: KeyEvent) {
        self.list.handle_key_event(key);
        self.channels.handle_key_event(key);
    }
}

impl FocusableComponent for Results {
    fn set_focus(&mut self, focus: bool) {
        self.list.set_focus(focus);
        self.channels.set_focus(focus);
        self.propagate_parental_focus(focus);
    }
}

impl ParentalFocusComponent for Results {
    fn propagate_parental_focus(&mut self, focus: bool) {
        self.list.propagate_parental_focus(focus);
        self.channels.propagate_parental_focus(focus);
    }
}
