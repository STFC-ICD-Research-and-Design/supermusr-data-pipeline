pub mod messages_received {
    use prometheus_client::encoding::{EncodeLabelSet, EncodeLabelValue};

    #[derive(Debug, Clone, Eq, Hash, PartialEq, EncodeLabelValue)]
    pub enum MessageKind {
        Trace,
        Event,
        Unknown,
    }

    #[derive(Debug, Clone, Eq, Hash, PartialEq, EncodeLabelSet)]
    pub struct MessagesReceivedLabels {
        kind: MessageKind,
    }

    impl MessagesReceivedLabels {
        pub fn new(kind: MessageKind) -> Self {
            Self { kind }
        }
    }
}

pub mod failures {
    use prometheus_client::encoding::{EncodeLabelSet, EncodeLabelValue};

    #[derive(Debug, Clone, Eq, Hash, PartialEq, EncodeLabelValue)]
    pub enum FailureKind {
        UnableToDecodeMessage,
        DataProcessingFailed,
        KafkaPublishFailed,
        FileWriteFailed,
    }

    #[derive(Debug, Clone, Eq, Hash, PartialEq, EncodeLabelSet)]
    pub struct FailureLabels {
        kind: FailureKind,
    }

    impl FailureLabels {
        pub fn new(kind: FailureKind) -> Self {
            Self { kind }
        }
    }
}
