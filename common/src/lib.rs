pub mod metrics;

pub type Time = u32;
pub type Channel = u32;
pub type Intensity = u16;

#[derive(Default)]
pub struct EventData {
    pub time: Vec<Time>,
    pub channel: Vec<Channel>,
    pub voltage: Vec<Intensity>,
}
