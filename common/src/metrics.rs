pub mod metric_names {
    pub const MESSAGES_PROCESSED: &str = "messages_processed";
    pub const FAILURES: &str = "failures";
    pub const MESSAGES_RECEIVED: &str = "messages_received";
}

pub mod messages_received {
    #[derive(Debug, Clone, Eq, Hash, PartialEq)]
    pub enum MessageKind {
        Trace,
        Event,
        Unknown,
    }

    // Label building function
    pub fn get_label(message_kind: MessageKind) -> (&'static str, &'static str) {
        (
            "message_kind",
            match message_kind {
                MessageKind::Trace => "trace",
                MessageKind::Event => "event",
                MessageKind::Unknown => "unknown",
            },
        )
    }
}

pub mod failures {
    #[derive(Debug, Clone, Eq, Hash, PartialEq)]
    pub enum FailureKind {
        UnableToDecodeMessage,
        DataProcessingFailed,
        KafkaPublishFailed,
        FileWriteFailed,
    }

    // Label building function
    pub fn get_label(failure_kind: FailureKind) -> (&'static str, &'static str) {
        (
            "failure_kind",
            match failure_kind {
                FailureKind::UnableToDecodeMessage => "unable_to_decode_message",
                FailureKind::DataProcessingFailed => "data_processing_failed",
                FailureKind::KafkaPublishFailed => "kafka_publish_failed",
                FailureKind::FileWriteFailed => "file_write_failed",
            },
        )
    }
}
