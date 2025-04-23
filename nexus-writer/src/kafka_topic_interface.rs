use rdkafka::{
    consumer::{Consumer, StreamConsumer},
    error::KafkaResult,
};
use tracing::debug;

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

fn get_nonrepeating_list(list: Vec<&str>) -> Vec<&str> {
    let mut topics_to_subscribe = list.into_iter().collect::<Vec<&str>>();
    debug!("{topics_to_subscribe:?}");
    topics_to_subscribe.sort();
    topics_to_subscribe.dedup();
    topics_to_subscribe
}

impl Topics {
    fn get_full_nonrepeating_list(&self) -> Vec<&str> {
        get_nonrepeating_list(vec![
            &self.control,
            &self.log,
            &self.frame_event,
            &self.sample_env,
            &self.alarm,
        ])
    }

    fn get_continuous_only_nonrepeating_list(&self) -> Vec<&str> {
        get_nonrepeating_list(vec![&self.control, &self.log, &self.frame_event])
    }
}

#[derive(PartialEq)]
pub(crate) enum TopicMode {
    Full,
    ConitinousOnly,
}

pub(crate) struct TopicSubscriber<'a> {
    mode: Option<TopicMode>,
    consumer: &'a StreamConsumer,
    full_list: Vec<&'a str>,
    continous_only_list: Vec<&'a str>,
}

impl<'a> TopicSubscriber<'a> {
    pub(crate) fn new(consumer: &'a StreamConsumer, topics: &'a Topics) -> Self {
        let full_list = topics.get_full_nonrepeating_list();
        let continous_only_list = topics.get_continuous_only_nonrepeating_list();
        Self {
            mode: None,
            consumer,
            full_list,
            continous_only_list,
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
