mod builder;
mod hdf5;
mod run;
mod run_parameters;

pub(crate) use builder::Nexus;
pub(crate) use run::Run;
pub(crate) use run::RunLike;
pub(crate) use run_parameters::RunParameters;

pub(crate) const TIMESTAMP_FORMAT: &str = "%Y-%m-%dT%H:%M:%S%.f%z";
pub(crate) const DATETIME_FORMAT: &str = "%Y-%m-%dT%H:%M:%S%z";

pub(crate) mod nexus_class {
    pub(crate) const ROOT: &str = "NX_root";
    pub(crate) const ENTRY: &str = "NXentry";
    pub(crate) const INSTRUMENT: &str = "NXinstrument";
    pub(crate) const PERIOD: &str = "NXperiod";
    pub(crate) const SOURCE: &str = "NXsource";
    pub(crate) const DETECTOR: &str = "NXdetector";
    pub(crate) const EVENT_DATA: &str = "NXevent_data";
}
