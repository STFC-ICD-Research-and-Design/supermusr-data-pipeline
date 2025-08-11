//! Responsible for allowing the user to perform a search on a particular topic.
//! 
//! The particulars of the search method is executed by calling various iterator methods on [Searcher].

mod iterators;
mod searcher;

pub(crate) use iterators::{BackstepIter, BinarySearchIter, ForwardSearchIter};
pub(crate) use searcher::{Searcher, SearcherError};
