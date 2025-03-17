use super::NexusSchematic;

pub(crate) struct EventData {
    
}

impl NexusSchematic for EventData {
    const CLASS: &str = "NXeventdata";

    fn build_new_group(this: hdf5::Group) -> crate::NexusWriterResult<super::NexusGroup<Self>> {
        todo!()
    }

    fn open_group(parent: hdf5::Group) -> crate::NexusWriterResult<super::NexusGroup<Self>> {
        todo!()
    }

    fn close_group(parent: hdf5::Group) -> crate::NexusWriterResult<()> {
        todo!()
    }
}