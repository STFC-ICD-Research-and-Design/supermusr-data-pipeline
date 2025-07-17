//! Handles flatbuffer messages.
mod borrowed_messages;
mod cache;
mod digitiser_messages;


use cfg_if::cfg_if;

pub(crate) use borrowed_messages::BorrowedMessageError;
pub(crate) use cache::Cache;

cfg_if! {
    if #[cfg(feature = "ssr")] {
        pub(crate) use borrowed_messages::{
            EventListMessage, FBMessage, TraceMessage,
        };
    }
}