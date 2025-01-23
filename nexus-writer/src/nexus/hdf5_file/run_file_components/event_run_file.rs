use crate::nexus::{
    hdf5_file::hdf5_writer::{AttributeExt, DatasetExt, GroupExt, HasAttributesExt},
    nexus_class as NX, NexusSettings,
};
use chrono::{DateTime, Utc};
use hdf5::{types::VarLenUnicode, Dataset, Group};
use supermusr_common::{Channel, Time};
use supermusr_streaming_types::aev2_frame_assembled_event_v2_generated::FrameAssembledEventListMessage;

#[derive(Debug)]
pub(crate) struct EventRun {
    offset: Option<DateTime<Utc>>,

    num_messages: usize,
    num_events: usize,

    //  Frames
    event_index: Dataset,
    event_time_zero: Dataset,
    period_number: Dataset,
    frame_number: Dataset,
    frame_complete: Dataset,
    running: Dataset,
    veto_flags: Dataset,
    //  Events
    event_id: Dataset,
    pulse_height: Dataset,
    event_time_offset: Dataset,
}

impl EventRun {
    #[tracing::instrument(skip_all, level = "trace")]
    pub(crate) fn new_event_runfile(
        parent: &Group,
        nexus_settings: &NexusSettings,
    ) -> anyhow::Result<Self> {
        let detector = parent.add_new_group_to("detector_1", NX::EVENT_DATA)?;

        let pulse_height = detector.create_resizable_dataset::<f64>(
            "pulse_height",
            0,
            nexus_settings.eventlist_chunk_size,
        )?;
        let event_id = detector.create_resizable_dataset::<Channel>(
            "event_id",
            0,
            nexus_settings.eventlist_chunk_size,
        )?;
        let event_time_offset = detector.create_resizable_dataset::<Time>(
            "event_time_offset",
            0,
            nexus_settings.eventlist_chunk_size,
        )?;
        event_time_offset.add_attribute_to("units", "ns")?;

        let event_index = detector.create_resizable_dataset::<u32>(
            "event_index",
            0,
            nexus_settings.framelist_chunk_size,
        )?;
        let event_time_zero = detector.create_resizable_dataset::<u64>(
            "event_time_zero",
            0,
            nexus_settings.framelist_chunk_size,
        )?;
        let period_number = detector.create_resizable_dataset::<u64>(
            "period_number",
            0,
            nexus_settings.framelist_chunk_size,
        )?;
        event_time_zero.add_attribute_to("units", "ns")?;

        let frame_number = detector.create_resizable_dataset::<u64>(
            "frame_number",
            0,
            nexus_settings.framelist_chunk_size,
        )?;

        let frame_complete = detector.create_resizable_dataset::<u64>(
            "is_frame_complete",
            0,
            nexus_settings.framelist_chunk_size,
        )?;

        let running = detector.create_resizable_dataset::<bool>(
            "running",
            0,
            nexus_settings.framelist_chunk_size,
        )?;

        let veto_flags = detector.create_resizable_dataset::<u16>(
            "veto_flag",
            0,
            nexus_settings.framelist_chunk_size,
        )?;

        Ok(Self {
            offset: None,
            num_events: 0,
            num_messages: 0,
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

    #[tracing::instrument(skip_all, level = "trace", err(level = "warn"))]
    pub(crate) fn open_event_runfile(parent: &Group) -> anyhow::Result<Self> {
        let detector = parent.get_group("detector_1")?;

        let pulse_height = detector.get_dataset("pulse_height")?;
        let event_id = detector.get_dataset("event_id")?;
        let event_time_offset = detector.get_dataset("event_time_offset")?;

        let event_index = detector.get_dataset("event_index")?;
        let event_time_zero = detector.get_dataset("event_time_zero")?;
        let period_number = detector.get_dataset("period_number")?;
        let frame_number = detector.get_dataset("frame_number")?;
        let frame_complete = detector.get_dataset("is_frame_complete")?;
        let running = detector.get_dataset("running")?;
        let veto_flags = detector.get_dataset("veto_flag")?;

        let offset: Option<DateTime<Utc>> = {
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

    #[tracing::instrument(skip_all, level = "trace", err(level = "warn"))]
    pub(crate) fn init(&mut self, offset: &DateTime<Utc>) -> anyhow::Result<()> {
        self.offset = Some(*offset);
        self.event_time_zero
            .add_attribute_to("offset", &offset.to_rfc3339())?;
        Ok(())
    }

    #[tracing::instrument(
        skip_all,
        level = "trace",
        fields(message_number, num_events),
        err(level = "warn")
    )]
    pub(crate) fn push_message_to_event_runfile(
        &mut self,
        message: &FrameAssembledEventListMessage,
    ) -> anyhow::Result<()> {
        tracing::Span::current().record("message_number", self.num_messages);

        // Fields Indexed By Frame
        self.event_index.append_slice(&[self.num_events])?;

        // Recalculate time_zero of the frame to be relative to the offset value
        // (set at the start of the run).
        let time_zero = self.get_time_zero(message)?;

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
            .unwrap_or_default()
            .iter()
            .collect::<Vec<_>>();

        let times = &message
            .time()
            .unwrap_or_default()
            .iter()
            .collect::<Vec<_>>();

        let channels = &message
            .channel()
            .unwrap_or_default()
            .iter()
            .collect::<Vec<_>>();

        self.pulse_height.append_slice(&intensities)?;
        self.event_time_offset.append_slice(&times)?;
        self.event_id.append_slice(&channels)?;

        self.num_events = total_events;
        self.num_messages += 1;

        tracing::Span::current().record("num_events", num_new_events);
        Ok(())
    }

    pub(crate) fn get_time_zero(
        &self,
        message: &FrameAssembledEventListMessage,
    ) -> anyhow::Result<u64> {
        let timestamp: DateTime<Utc> = (*message
            .metadata()
            .timestamp()
            .ok_or(anyhow::anyhow!("Message timestamp missing."))?)
        .try_into()?;

        // Recalculate time_zero of the frame to be relative to the offset value
        // (set at the start of the run).
        let time_zero = self
            .offset
            .and_then(|offset| (timestamp - offset).num_nanoseconds())
            .ok_or(anyhow::anyhow!("event_time_zero cannot be calculated."))?
            as u64;

        Ok(time_zero)
    }
}
