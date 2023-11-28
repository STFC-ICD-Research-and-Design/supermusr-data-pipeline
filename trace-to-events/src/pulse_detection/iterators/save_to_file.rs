use super::{Pulse, Temporal};
use std::{
    fmt::Display,
    fs::File,
    io::{Error, Write},
    path::Path,
};

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
    fn save_to_file(self, path: &Path) -> Result<(), Error>;
}

impl<I> SaveToFileFilter<I> for I
where
    I: Iterator,
    I::Item: SavablePoint,
{
    fn save_to_file(self, path: &Path) -> Result<(), Error> {
        let mut file = File::create(path)?;
        for item in self {
            item.write_to_file(&mut file)?;
        }
        Ok(())
    }
}
