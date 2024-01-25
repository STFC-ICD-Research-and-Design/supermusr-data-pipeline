mod eventlist;
mod histogramlist;

use anyhow::Result;
use chrono::{DateTime, Utc};
pub(crate) use eventlist::EventList;
use hdf5::Group;
use std::fmt::Debug;

pub(crate) trait InstanceType: Default + Debug + Clone {
    type MessageType<'a>: Debug;

    fn extract_message(data: &Self::MessageType<'_>) -> Result<Self>;
    fn timestamp(&self) -> &DateTime<Utc>;
}

pub(crate) trait ListType: Default + Debug {
    type MessageInstance: InstanceType;

    fn append_message(&mut self, data: Self::MessageInstance) -> Result<()>;
    fn write_hdf5(&self, parent: &Group) -> Result<()>;
}
