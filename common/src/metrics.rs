pub mod messages_received {
    pub const MESSAGE_KIND_TRACE: &str = "MessageKindTrace";
    pub const MESSAGE_KIND_EVENT: &str = "MessageKindEvent";
    pub const MESSAGE_KIND_UNKNOWN: &str = "MessageKindUnknown";
}

pub mod failures {
    pub const FAILURE_KIND_UNABLE_TO_DECODE_MESSAGE: &str = "FailureKindUnableToDecodeMessage";
    pub const FAILURE_KIND_DATA_PROCESSING_FAILED: &str = "FailureKindDataProcessingFailed";
    pub const FAILURE_KIND_KAFKA_PUBLISH_FAILED: &str = "FailureKindKafkaPublishFailed";
    pub const FAILURE_KIND_FILE_WRITE_FAILED: &str = "FailureKindFileWriteFailed";
}
