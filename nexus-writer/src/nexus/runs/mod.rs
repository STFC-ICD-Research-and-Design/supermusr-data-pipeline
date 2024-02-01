pub(super) mod parameters;

use super::messages::{InstanceType, ListType};
use anyhow::Result;
use hdf5::Group;
pub(super) use parameters::RunParameters;

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

    pub(super) fn repatriate_lost_messsages(
        &mut self,
        lost_messages: &mut Vec<L::MessageInstance>,
    ) -> Result<()> {
        log::debug!(
            "Repatriating upto {0} lost messages to Run with start: {1} and stop: {2}",
            lost_messages.len(),
            self.params.collect_from,
            self.params.collect_until.unwrap_or_default(),
        );
        for message in lost_messages
            .iter()
            .filter(|message| self.params.is_message_timestamp_valid(message.timestamp()))
        {
            self.lists.append_message(message.clone())?;
        }
        lost_messages
            .retain(|message| !self.params.is_message_timestamp_valid(message.timestamp()));
        log::debug!("{0} lost messages remaining", lost_messages.len());
        Ok(())
    }

    pub(super) fn write_hdf5(&self, parent: &Group, run_number: usize) -> Result<()> {
        let group = self.params.write_header(parent, run_number)?;
        self.lists.write_hdf5(&group)
    }
}
