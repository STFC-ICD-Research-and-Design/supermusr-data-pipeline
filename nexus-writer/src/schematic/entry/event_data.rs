use hdf5::{Dataset, Group};
use supermusr_common::{Channel, Time};

use crate::{
    hdf5_handlers::NexusHDF5Result, nexus::{ChunkSizeSettings, GroupExt},
    schematic::NexusSchematic,
};

mod labels {
    pub(super) const PULSE_HEIGHT: &str = "pulse_height";
    pub(super) const EVENT_ID: &str = "event_id";
    pub(super) const EVENT_TIME_ZERO: &str = "event_time_zero";
    pub(super) const EVENT_TIME_OFFSET: &str = "event_time_offset";
    pub(super) const EVENT_INDEX: &str = "event_index";
    pub(super) const PERIOD_NUMBER: &str = "period_number";
    pub(super) const FRAME_NUMBER: &str = "frame_number";
    pub(super) const FRAME_COMPLETE: &str = "frame_complete";
    pub(super) const RUNNING: &str = "running";
    pub(super) const VETO_FLAGS: &str = "veto_flags";
}

pub(crate) struct EventData {
    pulse_height: Dataset,
    event_id: Dataset,
    event_time_zero: Dataset,
    event_time_offset: Dataset,
    event_index: Dataset,
    period_number: Dataset,
    frame_number: Dataset,
    frame_complete: Dataset,
    running: Dataset,
    veto_flags: Dataset,
}

impl NexusSchematic for EventData {
    const CLASS: &str = "NXeventdata";
    type Settings = ChunkSizeSettings;

    fn build_group_structure(group: &Group, settings: &ChunkSizeSettings) -> NexusHDF5Result<Self> {
        Ok(Self {
            pulse_height: group.create_resizable_empty_dataset::<f64>(
                labels::PULSE_HEIGHT,
                settings.eventlist,
            )?,
            event_id: group.create_resizable_empty_dataset::<Channel>(
                labels::EVENT_ID,
                settings.eventlist,
            )?,
            event_time_zero: group.create_resizable_empty_dataset::<Time>(
                labels::EVENT_TIME_OFFSET,
                settings.eventlist,
            )?,
            event_time_offset: group.create_resizable_empty_dataset::<u32>(
                labels::EVENT_INDEX,
                settings.framelist,
            )?,
            event_index: group.create_resizable_empty_dataset::<u64>(
                labels::EVENT_TIME_ZERO,
                settings.framelist,
            )?,
            period_number: group.create_resizable_empty_dataset::<u64>(
                labels::PERIOD_NUMBER,
                settings.framelist,
            )?,
            frame_number: group.create_resizable_empty_dataset::<u64>(
                labels::FRAME_NUMBER,
                settings.framelist,
            )?,
            frame_complete: group.create_resizable_empty_dataset::<u64>(
                labels::FRAME_COMPLETE,
                settings.framelist,
            )?,
            running: group.create_resizable_empty_dataset::<bool>(
                labels::RUNNING,
                settings.framelist,
            )?,
            veto_flags: group.create_resizable_empty_dataset::<u16>(
                labels::VETO_FLAGS,
                settings.framelist,
            )?,
        })
    }

    fn populate_group_structure(group: &Group) -> NexusHDF5Result<Self> {
        todo!()
    }

    fn close_group() -> NexusHDF5Result<()> {
        todo!()
    }
}
