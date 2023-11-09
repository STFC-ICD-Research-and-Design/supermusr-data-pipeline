use std::fmt::{Debug, Display, Formatter, Result};

pub(crate) trait EventData: Default + Clone + Debug + Display {
    //fn make_event<T>(self, time: T) -> (T, Self) where T: Temporal {
    //    (time, self)
    //}
}

#[derive(Default, Clone, Copy, Debug, PartialEq)]
pub(crate) struct Empty {}
impl Display for Empty {
    fn fmt(&self, _f: &mut Formatter<'_>) -> Result {
        Ok(())
    }
}
impl EventData for Empty {}
