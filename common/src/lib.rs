pub mod metrics;

pub type DigitizerId = u8;
pub type Time = u32;
pub type Channel = u32;
pub type Intensity = u16;

pub type FrameNumber = u32;
pub type SampleRate = u64;

#[derive(Default)]
pub struct EventData {
    pub time: Vec<Time>,
    pub channel: Vec<Channel>,
    pub voltage: Vec<Intensity>,
}

pub const CHANNELS_PER_DIGITIZER: usize = 8;

pub fn channel_index(digitizer_index: usize, channel_index: usize) -> usize {
    (digitizer_index * CHANNELS_PER_DIGITIZER) + channel_index
}
