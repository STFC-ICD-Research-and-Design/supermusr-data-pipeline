use super::{builder::BuilderType, writer::add_new_slice_field_to};
use anyhow::{anyhow, Result};
use chrono::{DateTime, Duration, Utc};
use hdf5::Group;
use supermusr_common::{Channel, Time};
use supermusr_streaming_types::dev1_digitizer_event_v1_generated::DigitizerEventListMessage;

#[derive(Default)]
pub(crate) struct EventList {
    number_of_events: usize,
    // Indexed by event.
    event_time_offset: Vec<Time>,
    // Indexed by event.
    pulse_height: Vec<f64>,
    // Indexed by event.
    event_id: Vec<Channel>,
    // Indexed by frame.
    event_time_zero: Vec<i64>,
    // Indexed by frame.
    event_index: Vec<usize>,
    // Indexed by frame.
    period_number: Vec<u64>,
    // Indexed by frame.
    protons_per_pulse: Vec<u8>,
    // Indexed by frame.
    running: Vec<bool>,
    // Indexed by frame.
    frame_number: Vec<u32>,
    // Indexed by frame.
    veto_flags: Vec<u16>,
    // Unique to file
    offset: Option<DateTime<Utc>>,
}

impl BuilderType for EventList {
    type MessageType<'a> = DigitizerEventListMessage<'a>;

    fn process_message(&mut self, data: &Self::MessageType<'_>) -> Result<()> {
        self.event_time_zero.push({
            let timestamp = Into::<DateTime<Utc>>::into(
                *data
                    .metadata()
                    .timestamp()
                    .ok_or(anyhow!("Message timestamp missing."))?,
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
        });
        self.event_index.push(self.number_of_events);

        self.period_number.push(data.metadata().period_number());
        self.protons_per_pulse
            .push(data.metadata().protons_per_pulse());
        self.running.push(data.metadata().running());
        self.frame_number.push(data.metadata().frame_number());
        self.veto_flags.push(data.metadata().veto_flags());

        //  Number of Events
        let voltage = data.voltage().unwrap();
        let time = data.time().unwrap();
        let channel = data.channel().unwrap();
        if voltage.len() != time.len() || time.len() != channel.len() {
            // Error
        }
        self.number_of_events += voltage.len();
        //  Event Slices
        self.pulse_height.extend(voltage.iter().map(|v| v as f64));
        self.event_time_offset.extend(time.iter());
        self.event_id.extend(channel.iter());
        Ok(())
    }

    fn write_hdf5(&self, detector: &Group) -> Result<()> {
        add_new_slice_field_to(detector, "pulse_height", &self.pulse_height, &[])?;
        add_new_slice_field_to(detector, "event_id", &self.event_id, &[])?;
        add_new_slice_field_to(detector, "event_time_offset", &self.event_time_offset, &[])?;
        add_new_slice_field_to(
            detector,
            "event_time_zero",
            &self.event_time_zero,
            &[("offset", &self.offset.unwrap().to_string())],
        )?;
        add_new_slice_field_to(detector, "event_index", &self.event_index, &[])?;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn process_null() {}
    #[test]
    fn write_null() {}
}
