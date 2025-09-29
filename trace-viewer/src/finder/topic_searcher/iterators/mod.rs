//! These structures are created from various method of [Searcher].
//!
//! These methods consume [Searcher] and return an iterator which searches for and steps through
//! messages on the specified topic.
//! In each case, calling the [collect()] method returns a [Searcher] with the found messages.
//!
mod back_step;
mod binary;
mod forward;

pub(crate) use back_step::BackstepIter;
pub(crate) use binary::BinarySearchIter;
pub(crate) use forward::ForwardSearchIter;

use tracing::warn;
use crate::Timestamp;

/// Detects Chronologically Out of Order Timestamps in Adjacent Messages
#[derive(Default)]
struct NonChronologicalMessageDetector {
    /// The current timestamp.
    last_timestamp: Option<Timestamp>
}

impl NonChronologicalMessageDetector {
    /// Creates a new instance with a given timestamp, use `default()`
    /// if there should be no initial timestamp.
    fn new(last_timestamp: Timestamp) -> Self {
        Self { last_timestamp: Some(last_timestamp) }
    }

    /// Emits a warning if the given timestamp is less-than the current timestamp.
    /// Updates the current timestamp, to the given one.
    fn next(&mut self, next_timestamp: Timestamp) {
        self.last_timestamp
            .filter(|&last_timestamp|last_timestamp > next_timestamp)
            .inspect(|&last_timestamp|warn!("Timestamps Out of Order: {last_timestamp} > {next_timestamp}"));
        self.last_timestamp.replace(next_timestamp);
    }
}