use super::{builder::BuilderType, writer::add_new_slice_field_to};
use anyhow::{anyhow, Result};
use chrono::{DateTime, Duration, Utc};
use hdf5::Group;
use supermusr_common::{Channel, Intensity, Time};
use supermusr_streaming_types::dev1_digitizer_event_v1_generated::DigitizerEventListMessage;

#[derive(Default)]
pub(crate) struct EventList {
    // Indexed by event.
    event_time_offset: Vec<Time>,
    // Indexed by event.
    pulse_height: Vec<Intensity>,
    // Indexed by frame.
    event_time_zero: Vec<i64>,
    // Indexed by event.
    event_id: Vec<Channel>,
    // Indexed by frame.
    event_index: Vec<usize>,

    offset: Option<DateTime<Utc>>,
    number_of_events: usize,
}
impl BuilderType for EventList {
    type MessageType<'a> = DigitizerEventListMessage<'a>;

    fn process_message(&mut self, data: &Self::MessageType<'_>) -> Result<()> {
        self.event_time_zero.push(
            {
                let timestamp = Into::<DateTime<Utc>>::into(
                    *data
                    .metadata()
                    .timestamp()
                    .ok_or(anyhow!("Message timestamp missing."))?
                );
                if let Some(offset) = self.offset {
                    (timestamp - offset)
                        .num_nanoseconds()
                        .ok_or(anyhow!("event_time_zero cannot be calculated."))?
                } else {
                    self.offset = Some(timestamp);
                    Duration::zero()
                        .num_nanoseconds()
                        .ok_or(anyhow!("event_time_zero cannot be calculated."))?
                }
            }
        );
        self.event_index.push(self.number_of_events);

        //  Number of Events
        let voltage = data.voltage().unwrap();
        let time = data.time().unwrap();
        let channel = data.channel().unwrap();
        if voltage.len() != time.len() || time.len() != channel.len() {
            // Error
        }
        self.number_of_events += voltage.len();
        //  Event Slices
        self.pulse_height.extend(voltage.iter());
        self.event_time_offset.extend(time.iter());
        self.event_id.extend(channel.iter());
        Ok(())
    }

    fn write_hdf5(&self, detector: &Group) -> Result<()> {
        add_new_slice_field_to(
            &detector,
            "pulse_height",
            &self.pulse_height,
            &[("units", "mV")],
        )?;
        add_new_slice_field_to(&detector, "event_id", &self.event_id, &[])?;
        add_new_slice_field_to(&detector, "event_time_offset", &self.event_time_offset, &[])?;
        add_new_slice_field_to(
            &detector,
            "event_time_zero",
            &self.event_time_zero,
            &[
                ("offset", &self.offset.unwrap().to_string()),
                ("units", "ns"),
            ],
        )?;
        add_new_slice_field_to(&detector, "event_index", &self.event_index, &[])?;
        Ok(())
    }
}
