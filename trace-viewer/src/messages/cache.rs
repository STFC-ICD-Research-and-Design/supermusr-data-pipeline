//! Handles the storage of borrowed trace and eventlist flatbuffer messages.
use super::digitiser_messages::{DigitiserEventList, DigitiserMetadata, DigitiserTrace};
use cfg_if::cfg_if;
use serde::{Deserialize, Serialize};
use std::{collections::hash_map::HashMap, slice};

#[derive(Default, Debug, Clone)]
pub struct Cache {
    traces: HashMap<DigitiserMetadata, DigitiserTrace>,
    events: HashMap<DigitiserMetadata, DigitiserEventList>,
}
cfg_if! {
    if #[cfg(feature = "ssr")] {
        use super::digitiser_messages::FromMessage;
        use std::collections::hash_map::{Entry, self};
        use supermusr_streaming_types::{
            dat2_digitizer_analog_trace_v2_generated::DigitizerAnalogTraceMessage,
            dev2_digitizer_event_v2_generated::DigitizerEventListMessage,
        };
        use tracing::{error, info};

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
    }
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct VectorisedCache {
    traces: Vec<(DigitiserMetadata, DigitiserTrace)>,
    events: Vec<(DigitiserMetadata, DigitiserEventList)>,
}

impl VectorisedCache {
    pub(crate) fn iter(&self) -> slice::Iter<'_, (DigitiserMetadata, DigitiserTrace)> {
        self.traces.iter()
    }
}

impl Into<VectorisedCache> for Cache {
    fn into(self) -> VectorisedCache {
        VectorisedCache {
            traces: self.traces.into_iter().collect::<Vec<_>>(),
            events: self.events.into_iter().collect::<Vec<_>>(),
        }
    }
}
