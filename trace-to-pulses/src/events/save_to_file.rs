use std::{
    env,
    fs::{create_dir_all, File},
    io::{Error, Write},
};

//use tdengine::utils::log_then_panic_t;

use crate::{
    events::Event,
    pulse::Pulse,
    tracedata::{EventData, Temporal},
};

pub trait SaveEventsToFile<T, I, D>
where
    T: Temporal,
    I: Iterator<Item = Event<T, D>>,
    D: EventData,
{
    fn save_to_file(self, folder: &str, name: &str) -> Result<(), Error>;
}

fn create_file(folder: &str, name: &str) -> Result<File, Error> {
    let cd = env::current_dir()?;
    let path = cd.join(folder);
    create_dir_all(&path)?;
    File::create(path.join(name))
}

impl<T, I, D> SaveEventsToFile<T, I, D> for I
where
    T: Temporal,
    I: Iterator<Item = Event<T, D>>,
    D: EventData,
{
    fn save_to_file(self, folder: &str, name: &str) -> Result<(), Error> {
        let mut file = create_file(folder, name)?;
        for event in self {
            writeln!(&mut file, "{0},{1}", event.get_time(), event.get_data())?
        }
        Ok(())
    }
}

pub trait SavePulsesToFile<I>
where
    I: Iterator<Item = Pulse>,
{
    fn save_to_file(self, folder: &str, name: &str) -> Result<(), Error>;
}

impl<I> SavePulsesToFile<I> for I
where
    I: Iterator<Item = Pulse>,
{
    fn save_to_file(self, folder: &str, name: &str) -> Result<(), Error> {
        let mut file = create_file(folder, name)?;
        for pulse in self {
            writeln!(&mut file, "{pulse}")?;
        }
        Ok(())
    }
}
