use rdkafka::{
    consumer::{Consumer, StreamConsumer},
    error::KafkaResult,
};

#[derive(PartialEq)]
pub(crate) enum TopicMode {
    Full,
    ConitinousOnly,
}

pub(crate) trait KafkaTopicInterface {
    fn ensure_subscription_mode_is(&mut self, mode: TopicMode) -> KafkaResult<()>;
}

pub(super) struct Topics {
    pub(super) control: String,
    pub(super) log: String,
    pub(super) frame_event: String,
    pub(super) sample_env: String,
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
