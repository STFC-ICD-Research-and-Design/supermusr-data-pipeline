use chrono::{DateTime, Utc};
use anyhow::Result;
use hdf5::Group;
use supermusr_common::{Time, Intensity, Channel};
use supermusr_streaming_types::dev1_digitizer_event_v1_generated::DigitizerEventListMessage;
use super::{builder::BuilderType, writer::add_new_slice_field_to};

#[derive(Default)]
pub(crate) struct EventList {
    // Indexed by event.
    event_time_offset : Vec<Time>,
    // Indexed by event.
    pulse_height : Vec<Intensity>,
    // Indexed by frame.
    event_time_zero : Vec<Time>,
    // Indexed by event.
    event_id : Vec<Channel>,
    // Indexed by frame.
    event_index: Vec<usize>,

    offset: Option<DateTime<Utc>>,
    number_of_events : usize,
}
impl BuilderType for EventList {
    type MessageType<'a> = DigitizerEventListMessage<'a>;

    fn process_message (&mut self, data : &Self::MessageType<'_>) -> Result<()> {
        if let Some(offset) = self.offset {
            self.event_time_zero
                .push(
                    (Into::<DateTime<Utc>>::into(*data.metadata().timestamp().unwrap()) - offset)
                        .num_nanoseconds().unwrap() as Time
                );
        } else {
            self.offset = Some(Into::<DateTime<Utc>>::into(*data.metadata().timestamp().unwrap())); 
            self.event_time_zero.push(0);
        }
        self.event_index.push(self.number_of_events);
        if data.voltage().unwrap().len() != data.time().unwrap().len() || data.time().unwrap().len() != data.channel().unwrap().len()
        {
            // Error
        }
        self.number_of_events += data.voltage().unwrap().len();
        self.pulse_height.extend(data.voltage().unwrap().iter());
        self.event_time_offset.extend(data.time().unwrap().iter());
        self.event_id.extend(data.channel().unwrap().iter());
        Ok(())
    }

    fn write_hdf5(&self, detector : &Group) -> Result<()> {
        add_new_slice_field_to(&detector, "pulse_height", &self.pulse_height, &[("units", "mV")])?;
        add_new_slice_field_to(&detector, "event_id", &self.event_id, &[])?;
        add_new_slice_field_to(&detector, "event_time_offset", &self.event_time_offset, &[])?;
        add_new_slice_field_to(&detector, "event_time_zero", &self.event_time_zero, &[("offset", &self.offset.unwrap().to_string()), ("units","ns")])?;
        add_new_slice_field_to(&detector, "event_index", &self.event_index, &[])?;
        Ok(())
    }

}