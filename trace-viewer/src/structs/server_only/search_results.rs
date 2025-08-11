use crate::{app::server_functions::SessionError, structs::digitiser_messages::{FromMessage, DigitiserEventList, DigitiserMetadata, DigitiserTrace}};
use std::collections::{hash_map::{self, Entry}, HashMap};
use supermusr_streaming_types::{
    dat2_digitizer_analog_trace_v2_generated::DigitizerAnalogTraceMessage,
    dev2_digitizer_event_v2_generated::DigitizerEventListMessage,
};
use tracing::{error, info};

#[derive(Debug, Clone)]
pub(crate) enum SearchResults {
    Cancelled,
    Successful { cache: Cache },
}

impl SearchResults {
    pub fn cache(&self) -> Result<&Cache, SessionError> {
        match self {
            SearchResults::Cancelled => Err(SessionError::SearchCancelled),
            SearchResults::Successful { cache } => Ok(cache),
        }
    }
}

#[derive(Default, Debug, Clone)]
pub struct Cache {
    traces: HashMap<DigitiserMetadata, DigitiserTrace>,
    events: HashMap<DigitiserMetadata, DigitiserEventList>,
}

impl Cache {
    pub(crate) fn push_trace(&mut self, msg: &DigitizerAnalogTraceMessage<'_>) {
        info!("New Trace");
        let metadata = DigitiserMetadata {
            id: msg.digitizer_id(),
            timestamp: msg
                .metadata()
                .timestamp()
                .copied()
                .expect("Timestamp should exist.")
                .try_into()
                .unwrap(),
        };
        match self.traces.entry(metadata) {
            Entry::Occupied(occupied_entry) => {
                error!("Trace already found: {0:?}", occupied_entry.key());
            }
            Entry::Vacant(vacant_entry) => {
                vacant_entry.insert(DigitiserTrace::from_message(msg));
            }
        }
    }

    pub(crate) fn iter(&self) -> hash_map::Iter<'_, DigitiserMetadata, DigitiserTrace> {
        self.traces.iter()
    }

    pub(crate) fn push_events(&mut self, msg: &DigitizerEventListMessage<'_>) {
        let metadata = DigitiserMetadata {
            id: msg.digitizer_id(),
            timestamp: msg
                .metadata()
                .timestamp()
                .copied()
                .expect("Timestamp should exist.")
                .try_into()
                .unwrap(),
        };
        match self.events.entry(metadata) {
            Entry::Occupied(occupied_entry) => {
                error!("Event list already found: {0:?}", occupied_entry.key());
            }
            Entry::Vacant(vacant_entry) => {
                vacant_entry.insert(DigitiserEventList::from_message(msg));
            }
        }
    }

    pub(crate) fn attach_event_lists_to_trace(&mut self) {
        for (metadata, events) in &self.events {
            match self.traces.entry(metadata.clone()) {
                Entry::Occupied(mut occupied_entry) => {
                    info!("Found Trace for Events");
                    occupied_entry.get_mut().events = Some(events.clone());
                }
                Entry::Vacant(vacant_entry) => {
                    error!("Trace not found: {0:?}", vacant_entry.key());
                }
            }
        }
    }
}