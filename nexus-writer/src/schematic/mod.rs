use hdf5::{types::VarLenUnicode, Dataset, Group, Location};
use runlog::RunLog;
use selog::SELog;
use event_data::EventData;

use crate::{NexusWriterResult, nexus::{DatasetExt, GroupExt}};

mod event_data;
mod runlog;
mod selog;

pub(crate) trait NexusSchematic : Sized {
    const CLASS: &str;

    fn build_new_group(this: Group) -> NexusWriterResult<NexusGroup<Self>>;
    fn open_group(parent: Group) -> NexusWriterResult<NexusGroup<Self>>;
    fn close_group(parent: Group) -> NexusWriterResult<()>;
}

pub(crate) struct NexusGroup<S : NexusSchematic> {
    group: Group,
    schematic: S,
}

struct Root {
    //file.add_attribute_to("HDF5_version", "1.14.3")?; // Can this be taken directly from the nix package;
    //file.add_attribute_to("NeXus_version", "")?; // Where does this come from?
    //file.add_attribute_to("file_name", &file.filename())?; //  This should be absolutized at some point
    //file.add_attribute_to("file_time", Utc::now().to_string().as_str())?; //  This should be formatted, the nanoseconds are overkill.
    raw_data_1: NexusGroup<Entry>
}

impl NexusSchematic for Root {
    const CLASS: &str = "NX_root";
    
    fn build_new_group(parent: Group) -> NexusWriterResult<NexusGroup<Self>> {
        let root = parent.get_group("root")?;
        Ok(NexusGroup::<Self>{
            group: root.clone(),
            schematic: Self { 
                raw_data_1: Entry::build_new_group(root)?
            }
        })
    }

    fn open_group(parent: Group) -> NexusWriterResult<NexusGroup<Self>> {
        let root = parent.get_group("root")?;
        Ok(NexusGroup::<Self>{
            group: root.clone(),
            schematic: Self {
                raw_data_1: Entry::open_group(root)?
            }
        })
    }
    
    fn close_group(parent: Group) -> NexusWriterResult<()> {
        Ok(())
    }
}

struct Entry {
    idf_version: Dataset,
    definition: Dataset,
    program_name: Dataset,
    run_number: Dataset,
    experiment_identifier: Dataset,

    start_time: Dataset,
    end_time: Dataset,
    name: Dataset,
    title: Dataset,

    instrument_name: Dataset,

    run_logs: NexusGroup<RunLog>,

    source_name: Dataset,
    source_type: Dataset,
    source_probe: Dataset,

    period_number: Dataset,
    period_type: Dataset,

    selogs: NexusGroup<SELog>,

    detector_1: NexusGroup<EventData>,
}


impl NexusSchematic for Entry {
    const CLASS: &str = "NXentry";

    fn build_new_group(parent: Group) -> NexusWriterResult<NexusGroup<Self>> {
        let group = parent.get_group("raw_data_1")?;
        let schematic = Self {
            idf_version: group.create_constant_scalar_dataset::<i32>("IDF_version", &2)?,
            definition: group.create_constant_string_dataset("definition", "")?,
            program_name: group.create_scalar_dataset::<VarLenUnicode>("program_name")?,
            run_number: group.create_scalar_dataset::<u32>("run_number")?,
            experiment_identifier: group.create_scalar_dataset::<VarLenUnicode>("experiment_identifier")?,
            start_time: todo!(),
            end_time: todo!(),
            name: group.create_constant_string_dataset("name", "")?,
            title: group.create_constant_string_dataset("title", "")?,
            instrument_name: group.create_constant_string_dataset("instrument_name", "")?,
            run_logs: todo!(),
            source_name: todo!(),
            source_type: todo!(),
            source_probe: todo!(),
            period_number: todo!(),
            period_type: todo!(),
            selogs: todo!(),
            detector_1: todo!()
        };

        Ok(NexusGroup::<Self> {
            group,
            schematic
        })
    }

    fn open_group(parent: Group) -> NexusWriterResult<NexusGroup<Self>> {
        todo!()
    }
    
    fn close_group(parent: Group) -> NexusWriterResult<()> {
        todo!()
    }
}