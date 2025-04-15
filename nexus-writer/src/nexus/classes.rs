#[derive(strum::Display)]
pub(crate) enum NexusClass {
    #[strum(to_string = "NXentry")]
    Entry,
    #[strum(to_string = "NXevent_data")]
    EventData,
    #[strum(to_string = "NXgeometry")]
    Geometry,
    #[strum(to_string = "NXinstrument")]
    Instrument,
    #[strum(to_string = "NXperiod")]
    Period,
    #[strum(to_string = "NX_root")]
    Root,
    #[strum(to_string = "NXrunlog")]
    Runlog,
    #[strum(to_string = "IXselog")]
    Selog,
    #[strum(to_string = "IXseblock")]
    SelogBlock,
    #[strum(to_string = "NXsource")]
    Source,
    #[strum(to_string = "NXsample")]
    Sample,
    #[strum(to_string = "NXlog")]
    Log,
}
