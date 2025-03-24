use hdf5::{Dataset, Group};
use supermusr_common::{Channel, Time};
use supermusr_streaming_types::aev2_frame_assembled_event_v2_generated::FrameAssembledEventListMessage;

use crate::{
    error::FlatBufferMissingError,
    hdf5_handlers::{AttributeExt, ConvertResult, NexusHDF5Error, NexusHDF5Result},
    nexus::{DatasetUnitExt, NexusUnits},
    nexus_structure::{NexusMessageHandler, NexusSchematic},
    run_engine::{
        run_messages::{InitialiseNewNexusRun, PushFrameEventList},
        ChunkSizeSettings, DatasetExt, GroupExt, HasAttributesExt, NexusDateTime,
    },
};

mod labels {
    pub(super) const PULSE_HEIGHT: &str = "pulse_height";
    pub(super) const EVENT_ID: &str = "event_id";
    pub(super) const EVENT_TIME_ZERO: &str = "event_time_zero";
    pub(super) const EVENT_TIME_ZERO_OFFSET: &str = "offset";
    pub(super) const EVENT_TIME_OFFSET: &str = "event_time_offset";
    pub(super) const EVENT_INDEX: &str = "event_index";
    pub(super) const PERIOD_NUMBER: &str = "period_number";
    pub(super) const FRAME_NUMBER: &str = "frame_number";
    pub(super) const FRAME_COMPLETE: &str = "frame_complete";
    pub(super) const RUNNING: &str = "running";
    pub(super) const VETO_FLAGS: &str = "veto_flags";
}

pub(crate) struct EventData {
    num_messages: usize,
    num_events: usize,
    offset: Option<NexusDateTime>,
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
            num_messages: Default::default(),
            num_events: Default::default(),
            offset: None,
            pulse_height: group
                .create_resizable_empty_dataset::<f64>(labels::PULSE_HEIGHT, settings.eventlist)?,
            event_id: group
                .create_resizable_empty_dataset::<Channel>(labels::EVENT_ID, settings.eventlist)?,
            event_time_zero: group
                .create_resizable_empty_dataset::<Time>(
                    labels::EVENT_TIME_OFFSET,
                    settings.eventlist,
                )?
                .with_units(NexusUnits::Nanoseconds)?,
            event_time_offset: group
                .create_resizable_empty_dataset::<u32>(labels::EVENT_INDEX, settings.framelist)?
                .with_units(NexusUnits::Nanoseconds)?,
            event_index: group.create_resizable_empty_dataset::<u64>(
                labels::EVENT_TIME_ZERO,
                settings.framelist,
            )?,
            period_number: group
                .create_resizable_empty_dataset::<u64>(labels::PERIOD_NUMBER, settings.framelist)?,
            frame_number: group
                .create_resizable_empty_dataset::<u64>(labels::FRAME_NUMBER, settings.framelist)?,
            frame_complete: group.create_resizable_empty_dataset::<u64>(
                labels::FRAME_COMPLETE,
                settings.framelist,
            )?,
            running: group
                .create_resizable_empty_dataset::<bool>(labels::RUNNING, settings.framelist)?,
            veto_flags: group
                .create_resizable_empty_dataset::<u16>(labels::VETO_FLAGS, settings.framelist)?,
        })
    }

    fn populate_group_structure(group: &Group) -> NexusHDF5Result<Self> {
        let pulse_height = group.get_dataset("pulse_height")?;
        let event_id = group.get_dataset("event_id")?;
        let event_time_offset = group.get_dataset("event_time_offset")?;

        let event_index = group.get_dataset("event_index")?;
        let event_time_zero = group.get_dataset("event_time_zero")?;
        let period_number = group.get_dataset("period_number")?;
        let frame_number = group.get_dataset("frame_number")?;
        let frame_complete = group.get_dataset("is_frame_complete")?;
        let running = group.get_dataset("running")?;
        let veto_flags = group.get_dataset("veto_flag")?;

        let offset: Option<NexusDateTime> = {
            if let Ok(offset) = event_time_zero.get_attribute("offset") {
                Some(offset.get_datetime_from()?)
            } else {
                None
            }
        };

        Ok(Self {
            offset,
            num_messages: event_time_zero.size(),
            num_events: event_time_offset.size(),
            event_id,
            event_index,
            pulse_height,
            event_time_offset,
            event_time_zero,
            period_number,
            frame_number,
            frame_complete,
            running,
            veto_flags,
        })
    }

    fn close_group() -> NexusHDF5Result<()> {
        todo!()
    }
}

impl NexusMessageHandler<InitialiseNewNexusRun<'_>> for EventData {
    fn handle_message(
        &mut self,
        InitialiseNewNexusRun(parameters): &InitialiseNewNexusRun<'_>,
    ) -> NexusHDF5Result<()> {
        self.offset = Some(parameters.collect_from);
        self.event_time_zero.add_attribute_to(
            labels::EVENT_TIME_ZERO_OFFSET,
            &parameters.collect_from.to_rfc3339(),
        )?;
        Ok(())
    }
}

impl EventData {
    pub(crate) fn get_time_zero(
        &self,
        message: &FrameAssembledEventListMessage,
    ) -> NexusHDF5Result<i64> {
        let timestamp: NexusDateTime =
            (*message
                .metadata()
                .timestamp()
                .ok_or(NexusHDF5Error::new_flatbuffer_missing(
                    FlatBufferMissingError::Timestamp,
                ))?)
            .try_into()?;

        // Recalculate time_zero of the frame to be relative to the offset value
        // (set at the start of the run).
        let time_zero = self
            .offset
            .and_then(|offset| (timestamp - offset).num_nanoseconds())
            .ok_or(NexusHDF5Error::new_flatbuffer_timestamp_convert_to_nanoseconds())?;

        Ok(time_zero)
    }
}

impl NexusMessageHandler<PushFrameEventList<'_>> for EventData {
    fn handle_message(
        &mut self,
        PushFrameEventList(message): &PushFrameEventList<'_>,
    ) -> NexusHDF5Result<()> {
        // Fields Indexed By Frame
        self.event_index.append_slice(&[self.num_events])?;

        // Recalculate time_zero of the frame to be relative to the offset value
        // (set at the start of the run).
        let time_zero = self
            .get_time_zero(message)
            .err_dataset(&self.event_time_zero)?;

        self.event_time_zero.append_slice(&[time_zero])?;
        self.period_number
            .append_slice(&[message.metadata().period_number()])?;
        self.frame_number
            .append_slice(&[message.metadata().frame_number()])?;
        self.frame_complete.append_slice(&[message.complete()])?;

        self.running.append_slice(&[message.metadata().running()])?;

        self.veto_flags
            .append_slice(&[message.metadata().veto_flags()])?;

        // Fields Indexed By Event
        let num_new_events = message.channel().unwrap_or_default().len();
        let total_events = self.num_events + num_new_events;

        let intensities = &message
            .voltage()
            .ok_or(NexusHDF5Error::new_flatbuffer_missing(
                FlatBufferMissingError::Intensities,
            ))?
            .iter()
            .collect::<Vec<_>>();

        let times = &message
            .time()
            .ok_or(NexusHDF5Error::new_flatbuffer_missing(
                FlatBufferMissingError::Times,
            ))?
            .iter()
            .collect::<Vec<_>>();

        let channels = &message
            .channel()
            .ok_or(NexusHDF5Error::new_flatbuffer_missing(
                FlatBufferMissingError::Channels,
            ))?
            .iter()
            .collect::<Vec<_>>();

        self.pulse_height.append_slice(intensities)?;
        self.event_time_offset.append_slice(times)?;
        self.event_id.append_slice(channels)?;

        self.num_events = total_events;
        self.num_messages += 1;
        Ok(())
    }
}
