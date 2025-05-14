//! This module defines the [NexusClass] enum which encapsulates the textual "NX_class" hdf5 attribute.

/// Encapsulates the textual "NX_class" hdf5 attribute, which appears in each group, indicating its purpose.
/// Any struct `S` which implements [crate::nexus::NexusSchematic] must define a constant [NexusClass].
/// The purpose of this is to allow the handling of nexus classes to be factored out of the group structure
/// into the [crate::nexus::NexusGroup] object.
#[derive(strum::Display)]
pub(crate) enum NexusClass {
    /// The nexus class for the `Entry` group structure.
    #[strum(to_string = "NXentry")]
    Entry,
    /// The nexus class for the `EventData` group structure.
    #[strum(to_string = "NXevent_data")]
    EventData,
    /// The nexus class for the `Geometry` group structure.
    #[strum(to_string = "NXgeometry")]
    Geometry,
    /// The nexus class for the `Instrument` group structure.
    #[strum(to_string = "NXinstrument")]
    Instrument,
    /// The nexus class for the `Period` group structure.
    #[strum(to_string = "NXperiod")]
    Period,
    /// The nexus class for the `Root` group structure.
    #[strum(to_string = "NX_root")]
    Root,
    /// The nexus class for the `RunLog` group structure.
    #[strum(to_string = "NXrunlog")]
    Runlog,
    /// The nexus class for the `SELog` group structure.
    #[strum(to_string = "IXselog")]
    Selog,
    /// The nexus class for the `ValueLog` group structure.
    #[strum(to_string = "IXseblock")]
    SelogBlock,
    /// The nexus class for the `Source` group structure.
    #[strum(to_string = "NXsource")]
    Source,
    /// The nexus class for the `Sample` group structure.
    #[strum(to_string = "NXsample")]
    Sample,
    /// The nexus class for the `Log` group structure.
    #[strum(to_string = "NXlog")]
    Log,
}
