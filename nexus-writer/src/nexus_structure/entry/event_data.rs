use hdf5::{Dataset, Group};
use supermusr_common::{Channel, Time};
use supermusr_streaming_types::aev2_frame_assembled_event_v2_generated::FrameAssembledEventListMessage;

use crate::{
    error::FlatBufferMissingError,
    hdf5_handlers::{
        AttributeExt, ConvertResult, DatasetExt, GroupExt, HasAttributesExt, NexusHDF5Error,
        NexusHDF5Result,
    },
    nexus::{DatasetUnitExt, NexusClass, NexusUnits},
    nexus_structure::{NexusMessageHandler, NexusSchematic},
    run_engine::{
        run_messages::{InitialiseNewNexusRun, PushFrameEventList},
        EventChunkSize, FrameChunkSize, NexusDateTime,
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
    const CLASS: NexusClass = NexusClass::EventData;
    type Settings = (EventChunkSize, FrameChunkSize);

    fn build_group_structure(
        group: &Group,
        (event_chunk_size, frame_chunk_size): &Self::Settings,
    ) -> NexusHDF5Result<Self> {
        Ok(Self {
            num_messages: Default::default(),
            num_events: Default::default(),
            offset: None,
            pulse_height: group
                .create_resizable_empty_dataset::<f64>(labels::PULSE_HEIGHT, *event_chunk_size)?,
            event_id: group
                .create_resizable_empty_dataset::<Channel>(labels::EVENT_ID, *event_chunk_size)?,
            event_time_offset: group
                .create_resizable_empty_dataset::<u32>(
                    labels::EVENT_TIME_OFFSET,
                    *event_chunk_size,
                )?
                .with_units(NexusUnits::Nanoseconds)?,
            event_time_zero: group
                .create_resizable_empty_dataset::<Time>(labels::EVENT_TIME_ZERO, *frame_chunk_size)?
                .with_units(NexusUnits::Nanoseconds)?,
            event_index: group
                .create_resizable_empty_dataset::<u64>(labels::EVENT_INDEX, *frame_chunk_size)?,
            period_number: group
                .create_resizable_empty_dataset::<u64>(labels::PERIOD_NUMBER, *frame_chunk_size)?,
            frame_number: group
                .create_resizable_empty_dataset::<u64>(labels::FRAME_NUMBER, *frame_chunk_size)?,
            frame_complete: group
                .create_resizable_empty_dataset::<u64>(labels::FRAME_COMPLETE, *frame_chunk_size)?,
            running: group
                .create_resizable_empty_dataset::<bool>(labels::RUNNING, *frame_chunk_size)?,
            veto_flags: group
                .create_resizable_empty_dataset::<u16>(labels::VETO_FLAGS, *frame_chunk_size)?,
        })
    }

    fn populate_group_structure(group: &Group) -> NexusHDF5Result<Self> {
        let pulse_height = group.get_dataset(labels::PULSE_HEIGHT)?;
        let event_id = group.get_dataset(labels::EVENT_ID)?;
        let event_time_offset = group.get_dataset(labels::EVENT_TIME_OFFSET)?;

        let event_index = group.get_dataset(labels::EVENT_INDEX)?;
        let event_time_zero = group.get_dataset(labels::EVENT_TIME_ZERO)?;
        let period_number = group.get_dataset(labels::PERIOD_NUMBER)?;
        let frame_number = group.get_dataset(labels::FRAME_NUMBER)?;
        let frame_complete = group.get_dataset(labels::FRAME_COMPLETE)?;
        let running = group.get_dataset(labels::RUNNING)?;
        let veto_flags = group.get_dataset(labels::VETO_FLAGS)?;

        let offset = event_time_zero
            .get_attribute(labels::EVENT_TIME_ZERO_OFFSET)
            .ok()
            .map(|offset| offset.get_datetime())
            .transpose()?;

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
}

impl NexusMessageHandler<InitialiseNewNexusRun<'_>> for EventData {
    fn handle_message(
        &mut self,
        InitialiseNewNexusRun { parameters }: &InitialiseNewNexusRun<'_>,
    ) -> NexusHDF5Result<()> {
        self.offset = Some(parameters.collect_from);
        self.event_time_zero.add_attribute(
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
                .ok_or(FlatBufferMissingError::Timestamp)?)
            .try_into()?;

        // Recalculate time_zero of the frame to be relative to the offset value
        // (set at the start of the run).
        let time_zero = self
            .offset
            .and_then(|offset| (timestamp - offset).num_nanoseconds())
            .ok_or_else(|| {
                NexusHDF5Error::flatbuffer_timestamp_convert_to_nanoseconds(
                    timestamp - self.offset.unwrap(),
                )
            })?;

        Ok(time_zero)
    }
}

impl NexusMessageHandler<PushFrameEventList<'_>> for EventData {
    fn handle_message(
        &mut self,
        &PushFrameEventList { message }: &PushFrameEventList<'_>,
    ) -> NexusHDF5Result<()> {
        // Fields Indexed By Frame
        self.event_index.append_value(self.num_events)?;

        // Recalculate time_zero of the frame to be relative to the offset value
        // (set at the start of the run).
        let time_zero = self
            .get_time_zero(message)
            .err_dataset(&self.event_time_zero)?;

        self.event_time_zero.append_value(time_zero)?;
        self.period_number
            .append_value(message.metadata().period_number())?;
        self.frame_number
            .append_value(message.metadata().frame_number())?;
        self.frame_complete.append_value(message.complete())?;

        self.running.append_value(message.metadata().running())?;

        self.veto_flags
            .append_value(message.metadata().veto_flags())?;

        // Fields Indexed By Event
        let num_new_events = message.channel().unwrap_or_default().len();
        let total_events = self.num_events + num_new_events;

        let intensities = &message
            .voltage()
            .ok_or(FlatBufferMissingError::Intensities)?
            .iter()
            .collect::<Vec<_>>();

        let times = &message
            .time()
            .ok_or(FlatBufferMissingError::Times)?
            .iter()
            .collect::<Vec<_>>();

        let channels = &message
            .channel()
            .ok_or(FlatBufferMissingError::Channels)?
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
