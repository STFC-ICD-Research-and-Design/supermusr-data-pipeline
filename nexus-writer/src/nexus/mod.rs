mod builder;
mod messages;
mod runs;
mod writer;

pub(crate) use builder::Nexus;
pub(crate) use messages::EventList;
use runs::{Run, RunParameters};

pub(crate) mod NexusClass {
  pub(crate) const root : &str = "NX_root";
  pub(crate) const entry : &str = "NXentry";
  pub(crate) const instrument : &str = "NXinstrument";
  pub(crate) const period : &str = "NXperiod";
  pub(crate) const source : &str = "NXsource";
  pub(crate) const detector : &str = "NXdetector";
  pub(crate) const event_data : &str = "NXevent_data";
}
