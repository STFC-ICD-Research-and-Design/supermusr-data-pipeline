use kagiyama::{AlwaysReady, Watcher};
use lazy_static::lazy_static;
use prometheus_client::encoding::text::Encode;
use prometheus_client::metrics::{counter::Counter, family::Family};

pub(crate) fn register(watcher: &mut Watcher<AlwaysReady>) {
    let mut registery = watcher.metrics_registry();

    registery.register(
        "messages_received",
        "Messages received by type from incomming Kafka topic",
        Box::new(MESSAGES_RECEIVED.clone()),
    );

    registery.register(
        "file_write_failures",
        "Failures writing messages to file",
        Box::new(FILE_WRITE_FAILURES.clone()),
    );
}

#[derive(Clone, Eq, Hash, PartialEq, Encode)]
pub(crate) enum MessageKind {
    Trace,
    Event,
    Unknown,
}

#[derive(Clone, Eq, Hash, PartialEq, Encode)]
pub(crate) struct MessagesReceivedLabels {
    kind: MessageKind,
}

impl MessagesReceivedLabels {
    pub(crate) fn new(kind: MessageKind) -> Self {
        Self { kind }
    }
}

lazy_static! {
    pub(crate) static ref MESSAGES_RECEIVED: Family::<MessagesReceivedLabels, Counter> =
        Family::<MessagesReceivedLabels, Counter>::default();
    pub(crate) static ref FILE_WRITE_FAILURES: Counter = Counter::default();
}
