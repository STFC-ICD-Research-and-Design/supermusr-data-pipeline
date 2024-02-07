use super::{super::writer::{add_new_slice_field_to, set_attribute_list_to}, InstanceType, ListType};
use anyhow::{anyhow, Result};
use chrono::{DateTime, Duration, Utc};
use hdf5::Group;
use supermusr_common::{Channel, Time};
use supermusr_streaming_types::dev1_digitizer_event_v1_generated::DigitizerEventListMessage;

const TIMESTAMP_FORMAT : &str = "%Y-%m-%dT%H:%M:%S%.f%z";

#[derive(Default, Debug, Clone)]
pub(crate) struct EventMessage {
    number_of_events: usize,
    event_time_offset: Vec<Time>,
    pulse_height: Vec<f64>,
    event_id: Vec<Channel>,

    period_number: u64,
    protons_per_pulse: u8,
    running: bool,
    frame_number: u32,
    veto_flags: u16,

    timestamp: DateTime<Utc>,
}

impl InstanceType for EventMessage {
    type MessageType<'a> = DigitizerEventListMessage<'a>;

    fn extract_message(data: &Self::MessageType<'_>) -> Result<Self> {
        //  Number of Events
        let voltage = data
            .voltage()
            .ok_or(anyhow!("No voltage data in event list message."))?;
        let time = data
            .time()
            .ok_or(anyhow!("No time data in event list message."))?;
        let channel = data
            .channel()
            .ok_or(anyhow!("No channel data in event list message."))?;
        if voltage.len() != time.len() || time.len() != channel.len() {
            Err(anyhow!(
                "Mismatched vector lengths: voltage: {0}, time: {1}, channel: {2}",
                voltage.len(),
                time.len(),
                channel.len()
            ))
        } else {
            let timestamp = data
                .metadata()
                .timestamp()
                .ok_or(anyhow!("Message timestamp missing."))?;
            Ok(Self {
                number_of_events: voltage.len(),
                event_time_offset: time.iter().collect(),
                pulse_height: voltage.iter().map(|v| v as f64).collect(),
                event_id: channel.iter().collect(),
                period_number: data.metadata().period_number(),
                protons_per_pulse: data.metadata().protons_per_pulse(),
                running: data.metadata().running(),
                frame_number: data.metadata().frame_number(),
                veto_flags: data.metadata().veto_flags(),
                timestamp: Into::<DateTime<Utc>>::into(*timestamp),
            })
        }
    }

    fn timestamp(&self) -> &DateTime<Utc> {
        &self.timestamp
    }
}

#[derive(Default, Debug)]
pub(crate) struct EventList {
    number_of_events: usize,
    // Indexed by event.
    event_time_offset: Vec<Time>,
    // Indexed by event.
    pulse_height: Vec<f64>,
    // Indexed by event.
    event_id: Vec<Channel>,
    // Indexed by frame.
    event_time_zero: Vec<u64>,
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

impl ListType for EventList {
    type MessageInstance = EventMessage;

    fn append_message(&mut self, data: Self::MessageInstance) -> Result<()> {
        self.event_time_zero.push({
            if let Some(offset) = self.offset {
                (*data.timestamp() - offset)
                    .num_nanoseconds()
                    .ok_or(anyhow!("event_time_zero cannot be calculated."))? as u64
            } else {
                self.offset = Some(data.timestamp().clone());
                Duration::zero()
                    .num_nanoseconds()
                    .ok_or(anyhow!("event_time_zero cannot be calculated."))? as u64
            }
        });
        self.event_index.push(self.number_of_events);

        self.period_number.push(data.period_number);
        self.protons_per_pulse.push(data.protons_per_pulse);
        self.running.push(data.running);
        self.frame_number.push(data.frame_number);
        self.veto_flags.push(data.veto_flags);

        self.number_of_events += data.number_of_events;
        //  Event Slices
        self.pulse_height.extend(data.pulse_height);
        self.event_time_offset.extend(data.event_time_offset);
        self.event_id.extend(data.event_id);
        Ok(())
    }

    fn write_hdf5(&self, detector: &Group) -> Result<()> {
        //add_new_slice_field_to::<u32>(detector, "spectrum_index", &[0], &[])?;
        //add_new_slice_field_to::<u32>(detector, "data", &[], &[])?;

        add_new_slice_field_to(detector, "pulse_height", &self.pulse_height)?;
        add_new_slice_field_to(detector, "event_id", &self.event_id)?;
        add_new_slice_field_to(detector, "event_index", &self.event_index)?;
        
        let event_time_offset = add_new_slice_field_to(detector, "event_time_offset", &self.event_time_offset)?;
        set_attribute_list_to(&event_time_offset, &[("units", "ns")])?;
        

        // Note to self, please update writer.rs. Attributes are so infrequently added it is better
        // to remove them from the new field functions, and return the field instead. 
        let event_time_zero = add_new_slice_field_to(detector, "event_time_zero", &self.event_time_zero)?;
        set_attribute_list_to(&event_time_zero,
            &[  ("units", "ns"),
                ("offset",
                &self
                    .offset
                    .ok_or(anyhow!("Offset not set: {0:?}", self))?
                    .format(TIMESTAMP_FORMAT)
                    .to_string(),
            )]
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use supermusr_streaming_types::{dev1_digitizer_event_v1_generated::{finish_digitizer_event_list_message_buffer, root_as_digitizer_event_list_message, root_as_digitizer_event_list_message_with_opts, DigitizerEventListMessageArgs}, flatbuffers::FlatBufferBuilder};

    use super::*;

    #[test]
    fn process_null() {
        let mut list = EventList::default();
        let msg = EventMessage::default();
        assert_eq!(*msg.timestamp(), DateTime::<Utc>::default());

        list.append_message(msg).unwrap();
        assert_eq!(list.offset, Some(DateTime::<Utc>::default()));
        assert!(list.pulse_height.is_empty());
        assert!(list.event_time_offset.is_empty());
        assert!(list.event_id.is_empty());
        assert_eq!(list.number_of_events,0);
        assert_eq!(list.event_index, vec![0]);
        assert_eq!(list.event_time_zero, vec![0]);
        assert_eq!(list.frame_number, vec![0]);
        assert_eq!(list.period_number, vec![0]);
        assert_eq!(list.protons_per_pulse, vec![0]);
        assert_eq!(list.running, vec![false]);
    }
    #[test]
    fn process_one() {
        let mut list = EventList::default();
        let args = DigitizerEventListMessageArgs {
            digitizer_id: todo!(),
            metadata: todo!(),
            time: todo!(),
            voltage: todo!(),
            channel: todo!(),
        };
        let mut fbb = FlatBufferBuilder::new();
        let message = DigitizerEventListMessage::create(&mut fbb, &args);
        finish_digitizer_event_list_message_buffer(&mut fbb, message);
        let message = root_as_digitizer_event_list_message(fbb.finished_data()).unwrap();
        let msg = EventMessage::extract_message(&message).unwrap();

        assert_eq!(*msg.timestamp(), DateTime::<Utc>::default());
        
        list.append_message(msg).unwrap();
        assert_eq!(list.offset, Some(DateTime::<Utc>::default()));
        assert!(list.pulse_height.is_empty());
        assert!(list.event_time_offset.is_empty());
        assert!(list.event_id.is_empty());
        assert_eq!(list.number_of_events,0);
        assert_eq!(list.event_index, vec![0]);
        assert_eq!(list.event_time_zero, vec![0]);
        assert_eq!(list.frame_number, vec![0]);
        assert_eq!(list.period_number, vec![0]);
        assert_eq!(list.protons_per_pulse, vec![0]);
        assert_eq!(list.running, vec![false]);
    }
}
