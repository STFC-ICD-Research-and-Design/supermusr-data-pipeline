use anyhow::{anyhow, Result};
use streaming_types::dev1_digitizer_event_v1_generated::DigitizerEventListMessage;

#[derive(Default)]
pub(crate) struct EventData {
    pub(crate) time: Vec<u32>,
    pub(crate) channel: Vec<u32>,
    pub(crate) voltage: Vec<u16>,
}

impl EventData {
    pub(crate) fn push(&mut self, msg: &DigitizerEventListMessage) {
        if let Some(v) = msg.time() {
            self.time.extend_from_slice(v.safe_slice());
        }

        if let Some(v) = msg.channel() {
            self.channel.extend_from_slice(v.safe_slice());
        }

        if let Some(v) = msg.voltage() {
            self.voltage.extend_from_slice(v.safe_slice());
        }
    }

    pub(crate) fn validate(&self) -> Result<usize> {
        if self.time.len() != self.channel.len() || self.time.len() != self.voltage.len() {
            return Err(anyhow!(
                "Mismatch between number of times ({}), channels ({}) and voltages ({})",
                self.time.len(),
                self.channel.len(),
                self.voltage.len(),
            ));
        }

        Ok(self.time.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_data_validate_empty() {
        let data = EventData::default();
        assert_eq!(data.validate().ok(), Some(0));
    }

    #[test]
    fn test_event_data_validate() {
        let data = EventData {
            time: vec![1, 2, 3],
            channel: vec![0, 1, 0],
            voltage: vec![3, 5, 6],
        };
        assert_eq!(data.validate().ok(), Some(3));
    }

    #[test]
    fn test_event_data_validate_fail_missing_time() {
        let data = EventData {
            time: vec![1, 2],
            channel: vec![0, 1, 0],
            voltage: vec![3, 5, 6],
        };
        assert!(data.validate().is_err());
    }

    #[test]
    fn test_event_data_validate_fail_missing_channel() {
        let data = EventData {
            time: vec![1, 2, 3],
            channel: vec![0, 1],
            voltage: vec![3, 5, 6],
        };
        assert!(data.validate().is_err());
    }

    #[test]
    fn test_event_data_validate_fail_missing_voltage() {
        let data = EventData {
            time: vec![1, 2, 3],
            channel: vec![0, 1, 0],
            voltage: vec![3, 5],
        };
        assert!(data.validate().is_err());
    }
}
