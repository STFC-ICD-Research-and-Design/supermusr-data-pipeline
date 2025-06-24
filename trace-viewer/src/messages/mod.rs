//! Handles flatbuffer messages.
mod borrowed_messages;
mod cache;
mod digitiser_messages;

pub(crate) use borrowed_messages::{
    EventListMessage, FBMessage, TraceMessage,
};
pub(crate) use cache::Cache;
pub(crate) use digitiser_messages::{DigitiserMetadata, DigitiserTrace, EventList, Trace};
