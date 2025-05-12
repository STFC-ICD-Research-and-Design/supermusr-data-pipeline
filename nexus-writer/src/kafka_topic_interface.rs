//! Allows `NexusEngine` to switch the subscribed topics without having access to `StreamConsumer`.
use rdkafka::{
    consumer::{Consumer, StreamConsumer},
    error::KafkaResult,
};

/// Indicates which topics should be subscribed to.
#[derive(PartialEq)]
pub(crate) enum TopicMode {
    /// Indicates all topics.
    Full,
    /// Indicates continuous topics, that is all except those containing `SELog` and `Alerts`.
    ConitinousOnly,
}

/// Interface for types such as `NexusEngine` to change the list of topics subscribed to by the Kafka consumer.
pub(crate) trait KafkaTopicInterface {
    /// Implementations should switch the list of subscribed topics to those indicated by `mode`.
    /// This method should be idempotent, that is if the mode is already `mode`, it should change nothing.
    fn ensure_subscription_mode_is(&mut self, mode: TopicMode) -> KafkaResult<()>;
}

/// Contains the name of each Kafka topic the consumer may be interested in.
/// Note that these topics don't need to be distinct.
pub(super) struct Topics {
    /// Should contain `RunStart` and `RunStop` messages.
    pub(super) control: String,
    /// Should contain `RunLog` messages.
    pub(super) log: String,
    /// Should contain the event lists.
    pub(super) frame_event: String,
    /// Should contain `SELog` messages.
    pub(super) sample_env: String,
    /// Should contain `Alarm` messages.
    pub(super) alarm: String,
}

impl Topics {
    fn topics_for_mode(&self, mode: TopicMode) -> Vec<&str> {
        let mut list: Vec<&str> = match mode {
            TopicMode::Full => vec![
                &self.control,
                &self.log,
                &self.frame_event,
                &self.sample_env,
                &self.alarm,
            ],
            TopicMode::ConitinousOnly => {
                vec![&self.control, &self.log, &self.frame_event]
            }
        };
        list.sort();
        list.dedup();
        list
    }
}

pub(crate) struct TopicSubscriber<'a> {
    mode: Option<TopicMode>,
    consumer: &'a StreamConsumer,
    full_list: Vec<&'a str>,
    continous_only_list: Vec<&'a str>,
}

impl<'a> TopicSubscriber<'a> {
    pub(crate) fn new(consumer: &'a StreamConsumer, topics: &'a Topics) -> Self {
        Self {
            mode: None,
            consumer,
            full_list: topics.topics_for_mode(TopicMode::Full),
            continous_only_list: topics.topics_for_mode(TopicMode::ConitinousOnly),
        }
    }
}

impl KafkaTopicInterface for TopicSubscriber<'_> {
    fn ensure_subscription_mode_is(&mut self, mode: TopicMode) -> KafkaResult<()> {
        if self
            .mode
            .as_ref()
            .is_none_or(|this_mode| this_mode.eq(&mode))
        {
            if self.mode.is_some() {
                self.consumer.unsubscribe();
            }
            match mode {
                TopicMode::Full => self.consumer.subscribe(&self.full_list)?,
                TopicMode::ConitinousOnly => self.consumer.subscribe(&self.continous_only_list)?,
            };
            self.mode = Some(mode);
        }
        Ok(())
    }
}

#[cfg(test)]
pub(crate) struct NoKafka;

#[cfg(test)]
impl KafkaTopicInterface for NoKafka {
    fn ensure_subscription_mode_is(&mut self, _mode: TopicMode) -> KafkaResult<()> {
        Ok(())
    }
}
