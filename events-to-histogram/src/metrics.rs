use kagiyama::{
    prometheus::metrics::{counter::Counter, family::Family},
    AlwaysReady, Watcher,
};
use lazy_static::lazy_static;
pub(crate) use supermusr_common::metrics::{
    failures::{FailureKind, FailureLabels},
    messages_received::{MessageKind, MessagesReceivedLabels},
};

pub(crate) fn register(watcher: &Watcher<AlwaysReady>) {
    let mut registry = watcher.metrics_registry();

    let registry = registry.sub_registry_with_prefix("eventstohistogram");

    registry.register(
        "messages_processed",
        "Messages succesfully processed and published",
        MESSAGES_PROCESSED.clone(),
    );

    registry.register("failures", "Failures by type", FAILURES.clone());

    registry.register(
        "messages_received",
        "Messages received by type from incomming Kafka topic",
        MESSAGES_RECEIVED.clone(),
    );
}

lazy_static! {
    pub(crate) static ref MESSAGES_PROCESSED: Counter = Counter::default();
    pub(crate) static ref FAILURES: Family::<FailureLabels, Counter> =
        Family::<FailureLabels, Counter>::default();
    pub(crate) static ref MESSAGES_RECEIVED: Family::<MessagesReceivedLabels, Counter> =
        Family::<MessagesReceivedLabels, Counter>::default();
}
