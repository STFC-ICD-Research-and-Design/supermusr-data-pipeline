//!
mod cache;

use chrono::{DateTime, Utc};
use rdkafka::{message::BorrowedMessage, Message};
use std::collections::HashMap;
use supermusr_common::{Channel, DigitizerId, Intensity, Time};
use supermusr_streaming_types::{
    dat2_digitizer_analog_trace_v2_generated::{
        digitizer_analog_trace_message_buffer_has_identifier,
        root_as_digitizer_analog_trace_message, DigitizerAnalogTraceMessage,
    },
    dev2_digitizer_event_v2_generated::{
        digitizer_event_list_message_buffer_has_identifier, root_as_digitizer_event_list_message,
        DigitizerEventListMessage,
    },
};

pub(crate) use cache::Cache;

pub(crate) trait CreateFromMessage<M> {
    fn create_from_message(msg: &M) -> Self;
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

impl CreateFromMessage<DigitizerAnalogTraceMessage<'_>> for DigitiserTrace {
    fn create_from_message(msg: &DigitizerAnalogTraceMessage) -> Self {
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

impl CreateFromMessage<DigitizerEventListMessage<'_>> for DigitiserEventList {
    fn create_from_message(msg: &DigitizerEventListMessage) -> Self {
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

pub(crate) trait FBMessage<'a>: Sized {
    type UnpackedMessage;

    fn from_borrowed_message(message: BorrowedMessage<'a>) -> Option<Self>;
    fn get_unpacked_message(&'a self) -> Option<Self::UnpackedMessage>;
    fn timestamp(&self) -> DateTime<Utc>;
    fn digitiser_id(&self) -> DigitizerId;
    fn get_borrowed_message(&self) -> &BorrowedMessage<'a>;
}

pub(crate) struct TraceMessage<'a> {
    message: BorrowedMessage<'a>,
    timestamp: DateTime<Utc>,
    digitiser_id: DigitizerId,
}

impl<'a> TraceMessage<'a> {
    pub(crate) fn has_channel(&self, channel: Channel) -> bool {
        self.get_unpacked_message()
            .and_then(|d| d.channels())
            .and_then(|c| c.iter().find(|c| c.channel() == channel))
            .is_some()
    }
}

impl<'a> FBMessage<'a> for TraceMessage<'a> {
    type UnpackedMessage = DigitizerAnalogTraceMessage<'a>;

    fn get_unpacked_message(&'a self) -> Option<Self::UnpackedMessage> {
        self.message.unpack_trace_message()
    }

    fn get_borrowed_message(&self) -> &BorrowedMessage<'a> {
        &self.message
    }

    fn from_borrowed_message(message: BorrowedMessage<'a>) -> Option<Self> {
        let trace = message.unpack_trace_message()?;

        let timestamp = trace
            .metadata()
            .timestamp()
            .cloned()
            .map(TryInto::try_into)
            .map(Result::ok)
            .flatten()?;
        let digitiser_id = trace.digitizer_id();

        Some(Self {
            message,
            timestamp,
            digitiser_id,
        })
    }

    fn timestamp(&self) -> DateTime<Utc> {
        self.timestamp
    }

    fn digitiser_id(&self) -> DigitizerId {
        self.digitiser_id
    }
}

pub(crate) struct EventListMessage<'a> {
    message: BorrowedMessage<'a>,
    timestamp: DateTime<Utc>,
    digitiser_id: DigitizerId,
}

impl<'a> FBMessage<'a> for EventListMessage<'a> {
    type UnpackedMessage = DigitizerEventListMessage<'a>;

    fn get_unpacked_message(&'a self) -> Option<Self::UnpackedMessage> {
        self.message.unpack_event_list_message()
    }

    fn get_borrowed_message(&self) -> &BorrowedMessage<'a> {
        &self.message
    }

    fn from_borrowed_message(message: BorrowedMessage<'a>) -> Option<Self> {
        let evlist = message.unpack_event_list_message()?;

        let timestamp = evlist
            .metadata()
            .timestamp()
            .cloned()
            .map(TryInto::try_into)
            .map(Result::ok)
            .flatten()?;

        let digitiser_id = evlist.digitizer_id();

        Some(Self {
            message,
            timestamp,
            digitiser_id,
        })
    }

    fn timestamp(&self) -> DateTime<Utc> {
        self.timestamp
    }

    fn digitiser_id(&self) -> DigitizerId {
        self.digitiser_id
    }
}

pub(crate) trait UnpackMessage<'a> {
    fn unpack_trace_message(&'a self) -> Option<DigitizerAnalogTraceMessage<'a>>;
    fn unpack_event_list_message(&'a self) -> Option<DigitizerEventListMessage<'a>>;
}

impl<'a> UnpackMessage<'a> for BorrowedMessage<'a> {
    fn unpack_trace_message(&'a self) -> Option<DigitizerAnalogTraceMessage<'a>> {
        self.payload()
            .filter(|payload| digitizer_analog_trace_message_buffer_has_identifier(payload))
            .and_then(|payload| root_as_digitizer_analog_trace_message(payload).ok())
    }

    fn unpack_event_list_message(&'a self) -> Option<DigitizerEventListMessage<'a>> {
        self.payload()
            .filter(|payload| digitizer_event_list_message_buffer_has_identifier(payload))
            .and_then(|payload| root_as_digitizer_event_list_message(payload).ok())
    }
}
