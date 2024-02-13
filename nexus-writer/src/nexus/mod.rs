mod builder;
mod messages;
mod run_parameters;

pub(crate) use super::hdf5_writer;
pub(crate) use builder::Nexus;
pub(crate) use messages::{EventList, GenericEventMessage, ListType};
use run_parameters::RunParameters;

pub(crate) mod nexus_class {
    pub(crate) const ROOT: &str = "NX_root";
    pub(crate) const ENTRY: &str = "NXentry";
    pub(crate) const INSTRUMENT: &str = "NXinstrument";
    pub(crate) const PERIOD: &str = "NXperiod";
    pub(crate) const SOURCE: &str = "NXsource";
    pub(crate) const DETECTOR: &str = "NXdetector";
    pub(crate) const EVENT_DATA: &str = "NXevent_data";
}
