mod aggregated;
mod cache;
mod partial;

pub(crate) use aggregated::AggregatedFrame;
pub(crate) use cache::FrameCache;

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
