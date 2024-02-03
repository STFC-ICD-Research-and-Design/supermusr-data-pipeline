mod builder;
mod messages;
mod runs;
mod writer;

pub(crate) use builder::Nexus;
pub(crate) use messages::EventList;
use runs::{Run, RunParameters};

mod NexusClass {
  const root = "NX_root";
  const entry = "NXentry";
  const instrument = "NXinstrument";
  const period = "NXperiod";
  const entry = "NXsource";
  const detector = "NXdetector";
  const event_data = "NXevent_data";
}
