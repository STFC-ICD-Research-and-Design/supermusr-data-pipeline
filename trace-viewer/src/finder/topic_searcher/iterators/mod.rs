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
