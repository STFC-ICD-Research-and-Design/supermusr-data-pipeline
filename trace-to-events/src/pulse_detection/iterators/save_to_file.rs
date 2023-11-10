use std::{
    env,
    fmt::Display,
    fs::{create_dir_all, File},
    io::{Error, Write},
};

//use tdengine::utils::log_then_panic_t;

use super::Temporal;

fn create_file(folder: &str, name: &str) -> Result<File, Error> {
    let cd = env::current_dir()?;
    let path = cd.join(folder);
    create_dir_all(&path)?;
    File::create(path.join(name))
}



pub(crate) trait SavablePoint {
    fn write_to_file(&self, file : &mut File) -> Result<(), Error>;
}
impl<T, E> SavablePoint for (T,E) where T: Temporal, E: Display, {
    fn write_to_file(&self, file : &mut File) -> Result<(), Error> {
        writeln!(file, "{0},{1}", self.0, self.1)
    }
}

pub trait SaveToFileFilter<I>
where
    I: Iterator,
    I::Item : SavablePoint,
{
    fn save_to_file(self, folder: &str, name: &str) -> Result<(), Error>;
}

impl<I> SaveToFileFilter<I> for I
where
    I: Iterator,
    I::Item : SavablePoint,
{
    fn save_to_file(self, folder: &str, name: &str) -> Result<(), Error> {
        let mut file = create_file(folder, name)?;
        for item in self {
            item.write_to_file(&mut file);
        }
        Ok(())
    }
}
