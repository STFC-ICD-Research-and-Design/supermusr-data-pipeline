mod aggregated;
mod cache;
mod partial;

pub(crate) use aggregated::AggregatedFrame;
pub(crate) use cache::FrameCache;

pub(crate) enum RejectMessageError {
    IdAlreadyPresent,
    TimestampTooEarly,
}

impl Into<&'static str> for RejectMessageError {
    fn into(self) -> &'static str {
        match self {
            RejectMessageError::IdAlreadyPresent => "id_already_present",
            RejectMessageError::TimestampTooEarly => "timestamp_too_early",
        }
    }
}
