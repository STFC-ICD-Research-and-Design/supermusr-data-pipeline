//! Contains structs responsible for executing a particular search method.
mod binary_by_timestamp;
mod dragnet;

use crate::structs::Topics;
use rdkafka::consumer::StreamConsumer;
use std::marker::PhantomData;

pub(crate) use binary_by_timestamp::BinarySearchByTimestamp;
pub(crate) use dragnet::Dragnet;

pub(crate) trait TaskClass {}

pub(crate) struct SearchTask<'a, C: TaskClass> {
    consumer: &'a StreamConsumer,
    topics: &'a Topics,
    phantom: PhantomData<C>,
}

impl<'a, C: TaskClass> SearchTask<'a, C> {
    pub(crate) fn new(consumer: &'a StreamConsumer, topics: &'a Topics) -> Self {
        Self {
            consumer,
            topics,
            phantom: PhantomData,
        }
    }
}
