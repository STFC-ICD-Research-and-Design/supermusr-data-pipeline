#[cfg(test)]
mod mock_nexus_file;
mod nexus_file;
mod units;

use std::path::Path;
use hdf5::Group;

use crate::{hdf5_handlers::{ConvertResult, NexusHDF5Result}, run_engine::{
    run_messages::{
        InitialiseNewNexusStructure, PushAbortRunWarning, PushAlarm, PushFrameEventList,
        PushIncompleteFrameWarning, PushRunLogData, PushRunResumeWarning, PushRunStart,
        PushRunStop, PushSampleEnvironmentLog, SetEndTime,
    }, GroupExt, RunParameters
}, NexusSettings};

#[cfg(test)]
pub(crate) use mock_nexus_file::NexusNoFile;
pub(crate) use nexus_file::NexusFile;
pub(crate) use units::{DatasetUnitExt, NexusUnits};

pub(crate) trait NexusSchematic: Sized {
    const CLASS: &str;
    type Settings;

    fn build_group_structure(parent: &Group, settings: &Self::Settings) -> NexusHDF5Result<Self>;
    fn populate_group_structure(parent: &Group) -> NexusHDF5Result<Self>;

    fn build_new_group(
        parent: &Group,
        name: &str,
        settings: &Self::Settings,
    ) -> NexusHDF5Result<NexusGroup<Self>> {
        let group = parent
            .add_new_group_to(name, Self::CLASS)
            .err_group(parent)?;
        let schematic = Self::build_group_structure(&group, settings).err_group(parent)?;
        Ok(NexusGroup { group, schematic })
    }

    fn open_group(parent: &Group, name: &str) -> NexusHDF5Result<NexusGroup<Self>> {
        let group = parent.get_group(name).err_group(parent)?;
        let schematic = Self::populate_group_structure(&group).err_group(parent)?;
        Ok(NexusGroup { group, schematic })
    }
    fn close_group() -> NexusHDF5Result<()>;
}

pub(crate) trait NexusMessageHandler<M> {
    fn handle_message(&mut self, message: &M) -> NexusHDF5Result<()>;
}

pub(crate) struct NexusGroup<S: NexusSchematic> {
    group: Group,
    schematic: S,
}

impl<S: NexusSchematic> NexusGroup<S> {
    pub(crate) fn extract<M, F: Fn(&S) -> M>(&self, f: F) -> M {
        f(&self.schematic)
    }
}

impl<M, S> NexusMessageHandler<M> for NexusGroup<S>
where
    S: NexusSchematic + NexusMessageHandler<M>,
{
    fn handle_message(&mut self, message: &M) -> NexusHDF5Result<()> {
        self.schematic.handle_message(message)
    }
}

pub(crate) trait NexusFileInterface:
    Sized
    + for<'a> NexusMessageHandler<InitialiseNewNexusStructure<'a>>
    + for<'a> NexusMessageHandler<PushFrameEventList<'a>>
    + for<'a> NexusMessageHandler<PushRunLogData<'a>>
    + for<'a> NexusMessageHandler<PushRunStart<'a>>
    + for<'a> NexusMessageHandler<PushRunStop<'a>>
    + for<'a> NexusMessageHandler<PushSampleEnvironmentLog<'a>>
    + for<'a> NexusMessageHandler<PushAbortRunWarning<'a>>
    + for<'a> NexusMessageHandler<PushRunResumeWarning<'a>>
    + for<'a> NexusMessageHandler<PushIncompleteFrameWarning<'a>>
    + for<'a> NexusMessageHandler<PushAlarm<'a>>
    + for<'a> NexusMessageHandler<SetEndTime<'a>>
{
    fn build_new_file(file_path: &Path, nexus_settings: &NexusSettings) -> NexusHDF5Result<Self>;
    fn open_from_file(file_path: &Path) -> NexusHDF5Result<Self>;
    fn extract_run_parameters(&self) -> NexusHDF5Result<RunParameters>;
}



