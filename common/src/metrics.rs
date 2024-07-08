pub mod metric_names {
    pub const FAILURES: &str = "failures";
    pub const FRAMES_SENT: &str = "frames sent";
    pub const MESSAGES_PROCESSED: &str = "messages_processed";
    pub const MESSAGES_RECEIVED: &str = "messages_received";
}

pub mod messages_received {
    #[derive(Debug, Clone, Eq, Hash, PartialEq)]
    pub enum MessageKind {
        Alarm,
        Event,
        LogData,
        RunStart,
        RunStop,
        SampleEnvironmentData,
        Trace,
        Unexpected,
    }

    // Label building function
    pub fn get_label(message_kind: MessageKind) -> (&'static str, &'static str) {
        (
            "message_kind",
            match message_kind {
                MessageKind::Alarm => "alarm",
                MessageKind::Event => "event",
                MessageKind::LogData => "log_data",
                MessageKind::RunStart => "run_start",
                MessageKind::RunStop => "run_stop",
                MessageKind::SampleEnvironmentData => "sample_environment_data",
                MessageKind::Trace => "trace",
                MessageKind::Unexpected => "unexpected",
            },
        )
    }
}

pub mod failures {
    #[derive(Debug, Clone, Eq, Hash, PartialEq)]
    pub enum FailureKind {
        DataProcessingFailed,
        DeliveryFailed,
        FileWriteFailed,
        InvalidMetadata,
        KafkaPublishFailed,
        UnableToDecodeMessage,
    }

    // Label building function
    pub fn get_label(failure_kind: FailureKind) -> (&'static str, &'static str) {
        (
            "failure_kind",
            match failure_kind {
                FailureKind::DataProcessingFailed => "data_processing_failed",
                FailureKind::DeliveryFailed => "delivery_failed",
                FailureKind::FileWriteFailed => "file_write_failed",
                FailureKind::InvalidMetadata => "invalid_metadata",
                FailureKind::KafkaPublishFailed => "kafka_publish_failed",
                FailureKind::UnableToDecodeMessage => "unable_to_decode_message",
            },
        )
    }
}
