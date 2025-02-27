use super::{EventData, Temporal};
use std::fmt::Debug;

pub(crate) trait EventPoint: Debug + Clone {
    type TimeType: Temporal;
    type EventType: EventData;

    fn get_time(&self) -> Self::TimeType;
    fn get_data(&self) -> &Self::EventType;
}

impl<T, E> EventPoint for (T, E)
where
    T: Temporal,
    E: EventData,
{
    type TimeType = T;
    type EventType = E;

    fn get_time(&self) -> T {
        self.0
    }

    fn get_data(&self) -> &E {
        &self.1
    }
}
