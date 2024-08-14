mod engine;
mod hdf5_file;
mod run;
mod run_parameters;

pub(crate) use engine::{NexusEngine, NexusSettings};
pub(crate) use run::Run;
pub(crate) use run_parameters::RunParameters;

pub(crate) const DATETIME_FORMAT: &str = "%Y-%m-%dT%H:%M:%S%z";

pub(crate) mod nexus_class {
    pub(crate) const DETECTOR: &str = "NXdetector";
    pub(crate) const ENTRY: &str = "NXentry";
    pub(crate) const EVENT_DATA: &str = "NXevent_data";
    pub(crate) const INSTRUMENT: &str = "NXinstrument";
    pub(crate) const PERIOD: &str = "NXperiod";
    pub(crate) const ROOT: &str = "NX_root";
    pub(crate) const RUNLOG: &str = "NXrunlog";
    pub(crate) const SELOG: &str = "IXselog";
    pub(crate) const SELOG_BLOCK: &str = "IXseblock";
    pub(crate) const SOURCE: &str = "NXsource";
    pub(crate) const LOG: &str = "NXlog";
}
