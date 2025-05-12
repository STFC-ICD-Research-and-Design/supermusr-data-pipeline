//! Helpers for constructing fields of the [crate::nexus_structure] module.
mod classes;
mod file_interface;
mod logs;
mod units;

use crate::hdf5_handlers::{ConvertResult, GroupExt, NexusHDF5Result};
pub(crate) use classes::NexusClass;
#[cfg(test)]
pub(crate) use file_interface::NexusNoFile;
pub(crate) use file_interface::{NexusFile, NexusFileInterface};
use hdf5::Group;
pub(crate) use logs::{AlarmMessage, LogMessage};
pub(crate) use units::{DatasetUnitExt, NexusUnits};

pub(crate) const DATETIME_FORMAT: &str = "%Y-%m-%dT%H:%M:%S%z";

/// Types implementing this trait represent a NeXus group
/// # Constants
/// - CLASS: The NexusClass of the group. 
/// # Associated Types
/// - Settings
pub(crate) trait NexusSchematic: Sized {
    const CLASS: NexusClass;
    type Settings;

    /// Implementation should create an instance of Self with new structure created in `group`
    fn build_group_structure(group: &Group, settings: &Self::Settings) -> NexusHDF5Result<Self>;
    /// Creates a new instance of Self with structure populated from `group`
    fn populate_group_structure(group: &Group) -> NexusHDF5Result<Self>;

    /// Implementation should create an hdf5 group in `parent` and calls [NexusSchematic::build_group_structure] on it,
    /// then wraps the result in [NexusGroup]
    /// # Example
    fn build_new_group(
        parent: &Group,
        name: &str,
        settings: &Self::Settings,
    ) -> NexusHDF5Result<NexusGroup<Self>> {
        let group = parent
            .add_new_group(name, &Self::CLASS.to_string())
            .err_group(parent)?;
        let schematic = Self::build_group_structure(&group, settings).err_group(parent)?;
        Ok(NexusGroup { group, schematic })
    }

    /// Opens the named hdf5 group in `parent` and calls [Self::populate_group_structure] on it,
    /// then wraps the result in [NexusGroup]
    /// # Example
    fn open_group(parent: &Group, name: &str) -> NexusHDF5Result<NexusGroup<Self>> {
        let group = parent.get_group(name).err_group(parent)?;
        let schematic = Self::populate_group_structure(&group).err_group(parent)?;
        Ok(NexusGroup { group, schematic })
    }
}

pub(crate) trait NexusMessageHandler<M> {
    fn handle_message(&mut self, message: &M) -> NexusHDF5Result<()>;
}

/// This struct is a wrapper for a hdf5 group, handling both the creation, loading of the group and
/// handles any messages that are relevant to it or any of its subgroups.
/// The specifics of the group's structure is implemented by `S`, via the `NexusSchematic` trait.
/// Instances of `NexusGroup` are not constructed through `NexusGroup`'s own methods but by calling
/// `NexusSchematic::build_new_group` or `NexusSchematic::open_group`.
/// #Example
/// ```rust
/// struct MyNexusGroupType {
///     ...
/// }
/// 
/// impl NexusSchematic for MyNexusGroupType {
///     ...
/// }
/// 
/// let group : NexusGroup<MyNexusGroupType> = MyNexusGroupType::build_new_group(&parent_group, "my_group", ...)?;
/// ```
pub(crate) struct NexusGroup<S: NexusSchematic> {
    /// Handle to hdf5 `Group` that this object represents
    group: Group,
    /// Object implementing `NexusSchematic` that handles the structure of the group
    schematic: S,
}

impl<S: NexusSchematic> NexusGroup<S> {
    /// Given an existing hdf5 group, this associated function attempts to populate
    /// an instance of `S` from it.
    /// #Example
    pub(crate) fn open_from_existing_group(group: Group) -> NexusHDF5Result<Self> {
        let schematic = S::populate_group_structure(&group)?;
        Ok(Self { group, schematic })
    }

    /// Gets the hdf5 group's name.
    pub(crate) fn get_name(&self) -> String {
        self.group
            .name()
            .split("/")
            .last()
            .expect("split has at least one element, this should never fail")
            .to_owned()
    }

    /// Applies `f` to `self.schematic`, where `f` is a non-mutating function on `S`,
    /// with arbitrary return type. This is used to extract values from `self.schematic`.
    /// #Example
    /// ```rust
    /// struct MyNexusGroupType {
    ///     counter: i32;
    /// }
    /// 
    /// impl NexusSchematic for MyNexusGroupType {
    ///     ...
    /// }
    /// 
    /// let group : NexusGroup<MyNexusGroupType> = MyNexusGroupType::build_new_group(&parent_group, "my_group", ...)?;
    /// let counter_value : i32 = group.extract(|group: &MyNexusGroupType|group.counter);
    /// ```
    pub(crate) fn extract<M, F: Fn(&S) -> M>(&self, f: F) -> M {
        f(&self.schematic)
    }
}

impl<M, S> NexusMessageHandler<M> for NexusGroup<S>
where
    S: NexusSchematic + NexusMessageHandler<M>,
{
    /// This implementation propagates the message to `self.schematic`, this is possible
    /// as `S` is required to implement `NexusMessageHandler<M>`.
    #[tracing::instrument(skip_all, level = "debug", err(level = "warn"))]
    fn handle_message(&mut self, message: &M) -> NexusHDF5Result<()> {
        self.schematic
            .handle_message(message)
            .err_group(&self.group)
    }
}
