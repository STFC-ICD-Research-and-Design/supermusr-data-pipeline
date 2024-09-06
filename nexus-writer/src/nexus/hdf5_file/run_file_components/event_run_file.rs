use crate::nexus::{
    hdf5_file::{add_attribute_to, add_new_group_to, create_resizable_dataset},
    nexus_class as NX, NexusSettings,
};
use chrono::{DateTime, Duration, Utc};
use hdf5::{types::VarLenUnicode, Dataset, Group};
use ndarray::s;
use supermusr_common::{Channel, Time};
use supermusr_streaming_types::aev2_frame_assembled_event_v2_generated::FrameAssembledEventListMessage;
use tracing::debug;

#[derive(Debug)]
pub(crate) struct EventRun {
    offset: Option<DateTime<Utc>>,

    num_messages: usize,
    num_events: usize,

    //  Frames
    event_index: Dataset,
    event_time_zero: Dataset,
    period_number: Dataset,
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
        let detector = add_new_group_to(parent, "detector_1", NX::EVENT_DATA)?;

        let pulse_height = create_resizable_dataset::<f64>(
            &detector,
            "pulse_height",
            0,
            nexus_settings.eventlist_chunk_size,
        )?;
        let event_id = create_resizable_dataset::<Channel>(
            &detector,
            "event_id",
            0,
            nexus_settings.eventlist_chunk_size,
        )?;
        let event_time_offset = create_resizable_dataset::<Time>(
            &detector,
            "event_time_offset",
            0,
            nexus_settings.eventlist_chunk_size,
        )?;
        add_attribute_to(&event_time_offset, "units", "ns")?;

        let event_index = create_resizable_dataset::<u32>(
            &detector,
            "event_index",
            0,
            nexus_settings.framelist_chunk_size,
        )?;
        let event_time_zero = create_resizable_dataset::<u64>(
            &detector,
            "event_time_zero",
            0,
            nexus_settings.framelist_chunk_size,
        )?;
        let period_number = create_resizable_dataset::<u64>(
            &detector,
            "period_number",
            0,
            nexus_settings.framelist_chunk_size,
        )?;
        add_attribute_to(&event_time_zero, "units", "ns")?;

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
        })
    }

    #[tracing::instrument(skip_all, level = "trace", err(level = "warn"))]
    pub(crate) fn open_event_runfile(parent: &Group) -> anyhow::Result<Self> {
        let detector = parent.group("detector_1")?;

        let pulse_height = detector.dataset("pulse_height")?;
        let event_id = detector.dataset("event_id")?;
        let event_time_offset = detector.dataset("event_time_offset")?;

        let event_index = detector.dataset("event_index")?;
        let event_time_zero = detector.dataset("event_time_zero")?;
        let period_number = detector.dataset("period_number")?;

        let offset: Option<DateTime<Utc>> = {
            if let Ok(offset) = event_time_zero.attr("offset") {
                let offset: VarLenUnicode = offset.read_scalar()?;
                Some(offset.parse()?)
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
        })
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
        let next_message_slice = s![self.num_messages..(self.num_messages + 1)];
        self.event_index.resize(self.num_messages + 1)?;
        self.event_index
            .write_slice(&[self.num_events], next_message_slice)?;

        let timestamp: DateTime<Utc> = (*message
            .metadata()
            .timestamp()
            .ok_or(anyhow::anyhow!("Message timestamp missing."))?)
        .try_into()?;

        let time_zero = {
            if let Some(offset) = self.offset {
                debug!("Offset found");
                timestamp - offset
            } else {
                add_attribute_to(&self.event_time_zero, "offset", &timestamp.to_rfc3339())?;
                self.offset = Some(timestamp);
                debug!("New offset set");
                Duration::zero()
            }
        }
        .num_nanoseconds()
        .ok_or(anyhow::anyhow!("event_time_zero cannot be calculated."))?
            as u64;

        self.event_time_zero.resize(self.num_messages + 1)?;
        self.event_time_zero
            .write_slice(&[time_zero], next_message_slice)?;

        self.period_number.resize(self.num_messages + 1)?;
        self.period_number
            .write_slice(&[message.metadata().period_number()], next_message_slice)?;

        // Fields Indexed By Event
        let num_new_events = message.channel().unwrap_or_default().len();
        let total_events = self.num_events + num_new_events;
        let next_event_block_slice = s![self.num_events..total_events];

        self.pulse_height.resize(total_events)?;
        self.pulse_height.write_slice(
            &message
                .voltage()
                .unwrap_or_default()
                .iter()
                .collect::<Vec<_>>(),
            next_event_block_slice,
        )?;

        self.event_time_offset.resize(total_events)?;
        self.event_time_offset.write_slice(
            &message
                .time()
                .unwrap_or_default()
                .iter()
                .collect::<Vec<_>>(),
            next_event_block_slice,
        )?;

        self.event_id.resize(total_events)?;
        self.event_id.write_slice(
            &message
                .channel()
                .unwrap_or_default()
                .iter()
                .collect::<Vec<_>>(),
            next_event_block_slice,
        )?;

        self.num_events = total_events;
        self.num_messages += 1;

        tracing::Span::current().record("num_events", num_new_events);
        Ok(())
    }
}
