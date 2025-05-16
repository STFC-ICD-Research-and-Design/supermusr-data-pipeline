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

/// The format to use in the `start_time` and `end_time` NeXus file fields.
pub(crate) const DATETIME_FORMAT: &str = "%Y-%m-%dT%H:%M:%S%z";

/// Types implementing this trait becomes NeXus group structures.
///
/// Group structures are the objects which implement the NeXus specification for
/// a particular group. They are reponsible for building the group, populating it with
/// existing values, and handling any modifications.
/// The first two purposes are handled by [Self::build_group_structure] and [Self::populate_group_structure],
/// the latter by implementing [NexusMessageHandler] on the group structure.
pub(crate) trait NexusSchematic: Sized {
    /// The [NexusClass] of the group, defining the nexus class here as a constant,
    /// factors out the handling of the class into the [NexusGroup] struct, rather
    /// the group structure.
    const CLASS: NexusClass;
    /// Type allowing access to global settings used in creating and opening data.
    type Settings;

    /// Builds datasets, attributes, and subgroups conforming as members of the provided hdf5 group handle.
    /// # Parameters
    ///  - group: handle of the group in which to build the contents.
    ///  - settings: Instance of global settings used in creating and opening data.
    /// # Return
    /// Instance of `Self`, which has fields allowing access to any created datasets,
    /// subgroups and attributes which may be modified or accessed in the future.
    fn build_group_structure(group: &Group, settings: &Self::Settings) -> NexusHDF5Result<Self>;

    /// Loads datasets, attributes, and subgroups from the provided hdf5 group handle.
    /// # Parameters
    ///  - group: handle of the group in which to build the contents.
    /// # Return
    /// Instance of `Self`, which has fields allowing access to any created datasets,
    /// subgroups and attributes which may be modified or accessed in the future.
    fn populate_group_structure(group: &Group) -> NexusHDF5Result<Self>;

    /// Creates an hdf5 group in `parent` and initialises it using the implementation's
    /// [build_group_structure] method, then wraps the results in [NexusGroup].
    /// # Parameters
    ///  - parent: handle of the parent group (or file) this group should be built in.
    ///  - name: name of the new group.
    ///  - settings: Instance of global settings used in creating and opening data.
    /// # Return
    /// A [NexusGroup] instance wrapping the instance built with [build_group_structure].
    /// # Error
    /// Any errors are tagged with the relevant hdf5 path by [err_group].
    /// 
    /// [build_group_structure]: Self::build_group_structure.
    /// [err_group]: NexusHDF5Result::err_group
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

    /// Opens the named hdf5 group in `parent` and loads its contents via the implementation's
    /// [populate_group_structure], then wraps the result in [NexusGroup].
    /// # Parameters
    ///  - parent: hdf5 handle of the group (or file) this group should be built in.
    ///  - name: name of the new group.
    /// # Return
    /// A [NexusGroup] instance wrapping the instance built with [populate_group_structure].
    /// # Error
    /// Any errors are tagged with the relevant hdf5 path by [err_group].
    /// 
    /// [populate_group_structure]: Self::populate_group_structure.
    /// [err_group]: NexusHDF5Result::err_group
    fn open_group(parent: &Group, name: &str) -> NexusHDF5Result<NexusGroup<Self>> {
        let group = parent.get_group(name).err_group(parent)?;
        let schematic = Self::populate_group_structure(&group).err_group(parent)?;
        Ok(NexusGroup { group, schematic })
    }
}

/// Allows group structure objects to handle messages which provide instructions to modify the NeXus file.
///
/// Can be implemented for any type intended to pass messages into [crate::nexus_structure] objects.
/// In particular this can be implemented for types implementing [NexusSchematic], and the generic implementation
/// of [NexusMessageHandler] for [NexusGroup] means that messages passed to a [NexusGroup] instance are
/// automatically passed to the underlying schematic type.
/// # Example
/// ```rust
/// struct MyNexusGroupStructure {
///     ...
/// }
///
/// struct MyImportantMessage {
///     ...
/// }
///
/// impl NexusMessageHandler<MyImportantMessage> for MyNexusGroupStructure {
///     fn handle_message(&mut self, message: &M) -> NexusHDF5Result<()> {
///         ...
///     }
/// }
///
/// let group : NexusGroup<MyNexusGroupStructure> = MyNexusGroupStructure::build_new_group(&parent_group, "my_group", ...)?;
/// group.handle_message(MyImportantMessage {
///     ...
/// })?;
/// ```
pub(crate) trait NexusMessageHandler<M> {
    fn handle_message(&mut self, message: &M) -> NexusHDF5Result<()>;
}

/// This struct is a wrapper for a hdf5 group, handling both the creation, loading of the group and
/// handles any messages that are relevant to it or any of its subgroups.
///
/// The specifics of the group's structure is implemented by a group structure type `S`, which implements
/// the [NexusSchematic] trait. Instances of [NexusGroup] are not constructed through [NexusGroup]'s own
/// methods but by calling [NexusSchematic::build_new_group] or [NexusSchematic::open_group].
/// # Example
/// ```rust
/// struct MyNexusGroupStructure {
///     ...
/// }
///
/// impl NexusSchematic for MyNexusGroupStructure {
///     ...
/// }
///
/// let group : NexusGroup<MyNexusGroupStructure> = MyNexusGroupStructure::build_new_group(&parent_group, "my_group", ...)?;
/// ```
pub(crate) struct NexusGroup<S: NexusSchematic> {
    /// Handle to hdf5 `Group` that this object represents
    group: Group,
    /// Object implementing `NexusSchematic` that handles the structure of the group
    schematic: S,
}

impl<S: NexusSchematic> NexusGroup<S> {
    /// Given an existing hdf5 group, this associated function populates an instance of `S` from it
    /// and wraps it in a [NexusGroup] instance.
    /// Note that: this is not the preferred way of constructing [NexusGroup],
    /// instead [NexusSchematic::build_new_group] or [NexusSchematic::open_group]
    /// are preferred, but in situations where the number of subgroups and their names
    /// are not known in advance (such as in loading RunLogs or SELogs), then this method is available
    /// for when groups are loaded by iterator.
    /// # Example
    /// Suppose `parent: Group` and `S : NexusSchematic`, then
    /// ```rust
    /// let nexus_groups: Vec<NexusGroup<S>> = parent.groups()?
    ///     .into_iter()
    ///     .map(NexusGroup::<S>::open_from_existing_group)
    ///     .collect::<Result<Vec<_>, _>>()?,
    /// ```
    /// opens a vector containing all subgroups of `parent`.
    /// # Parameters
    ///  - group: group handle to use.
    /// # Return
    /// A [NexusGroup] instance wrapping the instance built with [NexusSchematic::populate_group_structure].
    pub(crate) fn open_from_existing_group(group: Group) -> NexusHDF5Result<Self> {
        let schematic = S::populate_group_structure(&group)?;
        Ok(Self { group, schematic })
    }

    /// Gets the hdf5 group's name.
    /// # Return
    /// A [String] initialised to the group's name.
    /// 
    /// Note that [hdf5::Location::name] returns the group's full path, this is
    /// decomposed by splitting on "/", and the final element returned.
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
    /// # Example
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
    /// # Parameters
    ///  - f: the function which extracts a value from the instance of `S`.
    pub(crate) fn extract<M, F: Fn(&S) -> M>(&self, f: F) -> M {
        f(&self.schematic)
    }
}

impl<M, S> NexusMessageHandler<M> for NexusGroup<S>
where
    S: NexusSchematic + NexusMessageHandler<M>,
{
    #[tracing::instrument(skip_all, level = "debug", err(level = "warn"))]
    fn handle_message(&mut self, message: &M) -> NexusHDF5Result<()> {
        self.schematic
            .handle_message(message)
            .err_group(&self.group)
    }
}
