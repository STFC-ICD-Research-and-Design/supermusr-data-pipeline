use super::{Pulse, Temporal};
use std::{
    fmt::Display,
    fs::{create_dir_all, File},
    io::{Error, Write},
    path::Path,
};

fn create_file(folder: &Path, name: &Path) -> Result<File, Error> {
    create_dir_all(folder)?;
    File::create(folder.join(name))
}

pub(crate) trait SavablePoint {
    fn write_to_file(&self, file: &mut File) -> Result<(), Error>;
}

impl<T, E> SavablePoint for (T, E)
where
    T: Temporal,
    E: Display,
{
    fn write_to_file(&self, file: &mut File) -> Result<(), Error> {
        writeln!(file, "{0},{1}", self.0, self.1)
    }
}

impl SavablePoint for Pulse {
    fn write_to_file(&self, file: &mut File) -> Result<(), Error> {
        writeln!(file, "{0}", self)
    }
}

pub(crate) trait SaveToFileFilter<I>
where
    I: Iterator,
    I::Item: SavablePoint,
{
    fn save_to_file(self, folder: &Path, name: &Path) -> Result<(), Error>;
}

impl<I> SaveToFileFilter<I> for I
where
    I: Iterator,
    I::Item: SavablePoint,
{
    fn save_to_file(self, folder: &Path, name: &Path) -> Result<(), Error> {
        let mut file = create_file(folder, name)?;
        for item in self {
            item.write_to_file(&mut file)?;
        }
        Ok(())
    }
}
