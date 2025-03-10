use hdf5::{Dataset, Group, Location};
use runlog::RunLog;
use selog::SELog;
use supermusr_common::EventData;

mod event_data;
mod runlog;
mod selog;

pub(crate) trait NexusSchematic {
    const CLASS: &str;

    fn build_new_schematic(this: Group) -> Self;
    fn open_schematic(parent: Group) -> Self;
}

pub(crate) struct NexusGroup<S : NexusSchematic> {
    group: Group,
    schematic: S,
}

pub(crate) trait NexusDataset {
    fn new_constant() -> Self;
}

struct Root {
    root: Group,
    //file.add_attribute_to("HDF5_version", "1.14.3")?; // Can this be taken directly from the nix package;
    //file.add_attribute_to("NeXus_version", "")?; // Where does this come from?
    //file.add_attribute_to("file_name", &file.filename())?; //  This should be absolutized at some point
    //file.add_attribute_to("file_time", Utc::now().to_string().as_str())?; //  This should be formatted, the nanoseconds are overkill.
    raw_data_1: Entry
}

impl NexusGroup for Root {
    const CLASS: &str = "NX_root";

    fn this_group(self) -> Group {
        self.root
    }

    fn create_new(parent: Group) -> Self {
        let root = parent.group()
        Self {
            root: 
            raw_data_1: Entry::create_new()
        }
    }

    fn open(parent: Group) -> Self {
        parent.group(name)
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

    run_logs: RunLog,

    source_name: Dataset,
    source_type: Dataset,
    source_probe: Dataset,

    period_number: Dataset,
    period_type: Dataset,

    selogs: SELog,

    detector_1: EventData,
}


impl NexusGroup for Entry {
    const CLASS: &str = "NXentry";

    fn create_new() -> Self {
        Self {
            idf_version: todo!(),
            definition: todo!(),
            program_name: todo!(),
            run_number: todo!(),
            experiment_identifier: todo!(),
            start_time: todo!(),
            end_time: todo!(),
            name: todo!(),
            title: todo!(),
            instrument_name: todo!(),
            run_logs: todo!(),
            source_name: todo!(),
            source_type: todo!(),
            source_probe: todo!(),
            period_number: todo!(),
            period_type: todo!(),
            selogs: todo!(),
            detector_1: todo!(),
        }
    }

    fn open(location: Location) -> Self {
        todo!()
    }
}