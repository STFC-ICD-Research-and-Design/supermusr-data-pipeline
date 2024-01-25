mod eventlist;
mod histogramlist;

use std::fmt::Debug;
pub(crate) use eventlist::EventList;
use chrono::{DateTime, Duration, Utc};
use hdf5::{file::File, Group};

pub(crate) trait InstanceType: Default + Debug {
    type MessageType<'a> : Debug;
    
    fn extract_message(&mut self, data: &Self::MessageType<'_>) -> Result<()>;
    fn timestamp(&self) -> Option<&DateTime<Utc>>;
}

pub(crate) trait ListType: Default where {
    type MessageInstance : InstanceType;

    fn append_message(&mut self, data: &Self::MessageInstance) -> Result<()>;
    fn write_hdf5(&self, parent: &Group) -> Result<()>;
}