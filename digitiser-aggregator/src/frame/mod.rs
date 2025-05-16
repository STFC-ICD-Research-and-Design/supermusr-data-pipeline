//! This module defines and implements the objects needed to represent
//! partial and complete frames, as well as the cache to store them.
//! The data stored in each frame is abstracted as a generic type and
//! defined in the `data` module.

mod aggregated;
mod cache;
mod partial;

pub(crate) use aggregated::AggregatedFrame;
pub(crate) use cache::FrameCache;

/// This enum represents the reason why a digitiser event list message is rejected
/// *Variants
/// - IdAlreadyPresent: If the frame has already encountered an event list from this digitiser
/// - TimestampTooEarly: If the event list's timestamp is before the `latest_timestamp_dispatched` field in the `FrameCache` instance
pub(crate) enum RejectMessageError {
    IdAlreadyPresent,
    TimestampTooEarly,
}

impl From<RejectMessageError> for &'static str {
    fn from(value: RejectMessageError) -> Self {
        match value {
            RejectMessageError::IdAlreadyPresent => "id_already_present",
            RejectMessageError::TimestampTooEarly => "timestamp_too_early",
        }
    }
}
