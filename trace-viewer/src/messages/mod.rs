//! Handles flatbuffer messages.
mod borrowed_messages;
mod cache;
mod digitiser_messages;

use cfg_if::cfg_if;

pub(crate) use cache::VectorisedCache;
pub(crate) use digitiser_messages::TraceWithEvents;

cfg_if! {
    if #[cfg(feature = "ssr")] {
        pub(crate) use cache::Cache;
        pub(crate) use borrowed_messages::{
            BorrowedMessageError,
            EventListMessage, FBMessage, TraceMessage,
        };
    }
}
