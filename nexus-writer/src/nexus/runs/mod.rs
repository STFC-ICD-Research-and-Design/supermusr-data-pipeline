pub(super) mod parameters;

use crate::hdf5_writer::{add_new_group_to, set_attribute_list_to, set_group_nx_class, Hdf5Writer};
use crate::nexus::nexus_class as NX;

use super::messages::{InstanceType, ListType};
use anyhow::Result;
use chrono::{DateTime, Duration, Utc};
use hdf5::Group;
pub(super) use parameters::RunParameters;
use supermusr_streaming_types::ecs_6s4t_run_stop_generated::RunStop;
use tracing::debug;

#[derive(Debug)]
pub(super) struct Run<L: ListType> {
    params: RunParameters,
    lists: L,
    time_completed: Option<DateTime<Utc>>,
}

impl<L: ListType> Run<L> {
    pub(super) fn new(params: RunParameters) -> Self {
        Self {
            params,
            lists: L::default(),
            time_completed: None,
        }
    }

    pub(super) fn parameters(&self) -> &RunParameters {
        &self.params
    }

    pub(super) fn is_ready_to_write(&self, now: &DateTime<Utc>, delay: &Duration) -> bool {
        self.time_completed.map(|time|*now - time > *delay).unwrap_or(false)
    }

    pub(crate) fn set_stop_if_valid(&mut self, data: RunStop<'_>) -> Result<()> {
        self.time_completed = Some(Utc::now());
        self.params.set_stop_if_valid(data)
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

        let found_messages = lost_messages
            .iter()
            .map(|message| Ok(
                self.params.is_message_timestamp_valid(message.timestamp())?
                    .then_some(message)
            ))
            .collect::<Result<Vec<_>>>()?
            .into_iter()
            .flatten();
            

        for message in found_messages {
            self.lists.append_message(message.clone())?;
        }

        // Note it is safe to call unwrap here as if any error were possible,
        // the method would already have returned
        lost_messages.retain(|message|
            !self.params.is_message_timestamp_valid(message.timestamp()).unwrap()
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
