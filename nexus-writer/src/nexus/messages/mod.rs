mod eventlist;
mod histogramlist;

use anyhow::Result;
use chrono::{DateTime, Utc};
pub(crate) use eventlist::{EventList, GenericEventMessage};
use std::fmt::Debug;

use super::hdf5_writer::Hdf5Writer;

pub(crate) trait InstanceType: Default + Debug + Clone {
    type MessageType<'a>: Debug;

    fn extract_message(data: &Self::MessageType<'_>) -> Result<Self>;
    fn timestamp(&self) -> &DateTime<Utc>;
}





pub(crate) trait ListType: Default + Debug + Hdf5Writer {
    type MessageInstance: InstanceType;

    fn append_message(&mut self, data: Self::MessageInstance) -> Result<()>;
    fn has_content(&self) -> bool;
}
