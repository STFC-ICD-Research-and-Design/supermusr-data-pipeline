
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use supermusr_common::{Channel, DigitizerId, Intensity, Time};
use supermusr_streaming_types::{
    dat2_digitizer_analog_trace_v2_generated::{
        DigitizerAnalogTraceMessage,
    },
    dev2_digitizer_event_v2_generated::{
        DigitizerEventListMessage
    },
};

/// Provides method for creating object from a generic message.
/// 
/// This trait is used instead of [From<&M>] so it can be implemented for [DigitizerEventListMessage],
/// which is an alias of [HashMap].
/// Rust does not allow foreign traits to be implemented on foreign types.
pub(crate) trait FromMessage<M> {
    /// Performs the same function as [From::from].
    fn from_message(msg: M) -> Self;
}

/// Timeseries of signal intensities. The time and value scaling is not stored here, so interpretation is owner dependent.
pub(crate) type Trace = Vec<Intensity>;

/// Bundles all metadata which uniquely defines each digitiser message.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub(crate) struct DigitiserMetadata {
    /// Unique to each digitiser.
    pub(crate) id: DigitizerId,
    /// Unique to each frame.
    pub(crate) timestamp: DateTime<Utc>,
}

/// Encapsulates all traces of a digitiser trace message.
#[derive(Clone)]
pub(crate) struct DigitiserTrace {
    ///
    pub(crate) traces: HashMap<Channel, Trace>,
    pub(crate) events: Option<DigitiserEventList>,
}

impl FromMessage<&DigitizerAnalogTraceMessage<'_>> for DigitiserTrace {
    fn from_message(msg: &DigitizerAnalogTraceMessage) -> Self {
        let pairs: Vec<(Channel, Trace)> = msg
            .channels()
            .unwrap()
            .iter()
            .map(|x| (x.channel(), x.voltage().unwrap().iter().collect()))
            .collect();
        let traces: HashMap<Channel, Trace> = HashMap::from_iter(pairs.into_iter());
        DigitiserTrace {
            traces,
            events: None,
        }
    }
}

#[derive(Clone, Copy)]
pub(crate) struct Event {
    pub(crate) time: Time,
    pub(crate) intensity: Intensity,
}
pub(crate) type EventList = Vec<Event>;
pub(crate) type DigitiserEventList = HashMap<Channel, EventList>;

impl FromMessage<&DigitizerEventListMessage<'_>> for DigitiserEventList {
    fn from_message(msg: &DigitizerEventListMessage) -> Self {
        let mut events = HashMap::<Channel, EventList>::new();
        for (idx, chnl) in msg.channel().unwrap().iter().enumerate() {
            events
                .entry(chnl)
                .or_insert(Default::default())
                .push(Event {
                    time: msg.time().unwrap().get(idx),
                    intensity: msg.voltage().unwrap().get(idx),
                })
        }
        events
    }
}
