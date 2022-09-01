pub mod messages_received {
    use prometheus_client::encoding::text::Encode;

    #[derive(Clone, Eq, Hash, PartialEq, Encode)]
    pub enum MessageKind {
        Trace,
        Event,
        Unknown,
    }

    #[derive(Clone, Eq, Hash, PartialEq, Encode)]
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
    use prometheus_client::encoding::text::Encode;

    #[derive(Clone, Eq, Hash, PartialEq, Encode)]
    pub enum FailureKind {
        UnableToDecodeMessage,
        DataProcessingFailed,
        KafkaPublishFailed,
        FileWriteFailed,
    }

    #[derive(Clone, Eq, Hash, PartialEq, Encode)]
    pub struct FailureLabels {
        kind: FailureKind,
    }

    impl FailureLabels {
        pub fn new(kind: FailureKind) -> Self {
            Self { kind }
        }
    }
}
