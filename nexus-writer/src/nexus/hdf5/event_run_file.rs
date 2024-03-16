use super::{add_attribute_to, add_new_group_to, create_resizable_dataset};
use crate::{
    event_message::GenericEventMessage,
    nexus::{nexus_class as NX, TIMESTAMP_FORMAT},
};
use anyhow::{anyhow, Result};
use chrono::{DateTime, Duration, Utc};
use hdf5::{types::VarLenUnicode, Dataset, Group};
use ndarray::s;
use supermusr_common::{Channel, Time};
use tracing::debug;

#[derive(Debug)]
pub(super) struct EventRun {
    offset: Option<DateTime<Utc>>,

    num_messages: usize,
    num_events: usize,

    //  Frames
    event_index: Dataset,
    event_time_zero: Dataset,
    //  Events
    event_id: Dataset,
    pulse_height: Dataset,
    event_time_offset: Dataset,
}

impl EventRun {
    pub(crate) fn new(parent: &Group) -> Result<Self> {
        let detector = add_new_group_to(parent, "detector_1", NX::EVENT_DATA)?;

        let pulse_height = create_resizable_dataset::<f64>(&detector, "pulse_height", 0, 1024)?;
        let event_id = create_resizable_dataset::<Channel>(&detector, "event_id", 0, 1024)?;
        let event_time_offset =
            create_resizable_dataset::<Time>(&detector, "event_time_offset", 0, 1024)?;
        add_attribute_to(&event_time_offset, "units", "ns")?;

        let event_index = create_resizable_dataset::<u32>(&detector, "event_index", 0, 64)?;
        let event_time_zero = create_resizable_dataset::<u64>(&detector, "event_time_zero", 0, 64)?;
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
        })
    }

    pub(crate) fn open(parent: &Group) -> Result<Self> {
        let detector = parent.group("detector_1")?;

        let pulse_height = detector.dataset("pulse_height")?;
        let event_id = detector.dataset("event_id")?;
        let event_time_offset = detector.dataset("event_time_offset")?;

        let event_index = detector.dataset("event_index")?;
        let event_time_zero = detector.dataset("event_time_zero")?;

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
        })
    }

    pub(crate) fn push_message(&mut self, message: &GenericEventMessage) -> Result<()> {
        self.event_index.resize(self.num_messages + 1).unwrap();
        self.event_index.write_slice(
            &[self.num_events],
            s![self.num_messages..(self.num_messages + 1)],
        )?;

        let timestamp: DateTime<Utc> = (*message
            .metadata
            .timestamp()
            .ok_or(anyhow!("Message timestamp missing."))?)
        .try_into()?;

        let time_zero = {
            if let Some(offset) = self.offset {
                debug!("Offset found");
                timestamp - offset
            } else {
                add_attribute_to(
                    &self.event_time_zero,
                    "offset",
                    &timestamp.format(TIMESTAMP_FORMAT).to_string(),
                )?;
                self.offset = Some(timestamp);
                debug!("New offset set");
                Duration::zero()
            }
        }
        .num_nanoseconds()
        .ok_or(anyhow!("event_time_zero cannot be calculated."))? as u64;
        self.event_time_zero.resize(self.num_messages + 1)?;
        self.event_time_zero
            .write_slice(&[time_zero], s![self.num_messages..(self.num_messages + 1)])?;

        let num_new_events = message.channel.unwrap_or_default().len();
        let total_events = self.num_events + num_new_events;

        self.pulse_height.resize(total_events)?;
        self.pulse_height.write_slice(
            &message
                .voltage
                .unwrap_or_default()
                .iter()
                .collect::<Vec<_>>(),
            s![self.num_events..total_events],
        )?;

        self.event_time_offset.resize(total_events)?;
        self.event_time_offset.write_slice(
            &message.time.unwrap_or_default().iter().collect::<Vec<_>>(),
            s![self.num_events..total_events],
        )?;

        self.event_id.resize(total_events)?;
        self.event_id.write_slice(
            &message
                .channel
                .unwrap_or_default()
                .iter()
                .collect::<Vec<_>>(),
            s![self.num_events..total_events],
        )?;

        self.num_events = total_events;
        self.num_messages += 1;
        Ok(())
    }
}
