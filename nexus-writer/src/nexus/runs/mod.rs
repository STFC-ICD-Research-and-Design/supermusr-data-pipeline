pub(super) mod parameters;

use crate::hdf5_writer::{add_new_group_to, set_attribute_list_to, set_group_nx_class, Hdf5Writer};
use crate::nexus::nexus_class as NX;

use super::messages::{InstanceType, ListType};
use anyhow::Result;
use chrono::{DateTime, Utc};
use hdf5::Group;
pub(super) use parameters::RunParameters;
use tracing::debug;

pub(super) struct Run<L: ListType> {
    params: RunParameters,
    lists: L,
}

impl<L: ListType> Run<L> {
    pub(super) fn new(params: RunParameters) -> Self {
        Self {
            params,
            lists: L::default(),
        }
    }
    pub(super) fn parameters_mut(&mut self) -> &mut RunParameters {
        &mut self.params
    }

    pub(super) fn lists_mut(&mut self) -> &mut L {
        &mut self.lists
    }

    pub(super) fn parameters(&self) -> &RunParameters {
        &self.params
    }
    
    pub(crate) fn is_message_timestamp_valid(&self, timestamp: &DateTime<Utc>) -> Result<bool> {
        self.params.is_message_timestamp_valid(timestamp)
    }

    pub(super) fn repatriate_lost_messsages(
        &mut self,
        lost_messages: &mut Vec<L::MessageInstance>,
    ) -> Result<()> {
        debug!(
            "Repatriating upto {0} lost messages to Run with start: {1} and stop: {2}",
            lost_messages.len(),
            self.params.collect_from,
            self.params.collect_until.unwrap_or_default(),
        );
        for message in lost_messages
            .iter()
            .filter_map(|message: &<L as ListType>::MessageInstance|
                match self.is_message_timestamp_valid(message.timestamp()) {
                    Ok(true) => Some(Ok(message)),
                    Ok(false) => None,
                    Err(e) => Some(Err(e))
                }
            ).collect::<Result<Vec<_>>>()? {
            self.lists.append_message(message.clone())?;
        }

        // Note it is safe to call unwrap here as if any error were possible,
        // the method would already have returned
        lost_messages.retain(|message|
            !self.is_message_timestamp_valid(message.timestamp()).unwrap()
        );
        debug!("{0} lost messages remaining", lost_messages.len());
        Ok(())
    }
}

impl<L: ListType> Hdf5Writer for Run<L> {
    fn write_hdf5(&self, parent: &Group) -> Result<()> {
        set_group_nx_class(parent, NX::ROOT)?;

        set_attribute_list_to(
            parent,
            &[
                ("HDF5_version", "1.14.3"), // Can this be taken directly from the nix package?
                ("NeXus_version", ""),      // Where does this come from?
                ("file_name", &parent.filename()), //  This should be absolutized at some point
                ("file_time", Utc::now().to_string().as_str()), //  This should be formatted, the nanoseconds are overkill.
            ],
        )?;

        let entry = add_new_group_to(parent, "raw_data_1", NX::ENTRY)?;
        self.params.write_hdf5(&entry)?;

        let event_data = add_new_group_to(&entry, "detector_1", NX::EVENT_DATA)?;
        self.lists.write_hdf5(&event_data)
    }
}
