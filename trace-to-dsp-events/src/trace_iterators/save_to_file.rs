
use std::{fs::File, io::{Error, Write}, env, fmt::Display};

use tdengine::utils::log_then_panic_t;

use crate::{EventIter, Detector};


pub trait SaveToFile<I> where I : Iterator {
    fn save_to_file(self, name : &str) -> Result<(),Error>;
}

impl<I,T : Display,V : Display> SaveToFile<I> for I where I: Iterator<Item = (T,V)> {
    fn save_to_file(self, name : &str) -> Result<(),Error> {
        let cd = env::current_dir().unwrap_or_else(|e|log_then_panic_t(format!("Cannot obtain current directory : {e}")));
        let mut file = File::create(cd.join(name)).unwrap_or_else(|e|log_then_panic_t(format!("Cannot create {name} file : {e}")));
        for (index,value) in self {
            writeln!(&mut file,"{index},{value}").unwrap_or_else(|e|log_then_panic_t(format!("Cannot write to {name} file : {e}")))
        }
        Ok(())
    }
}




impl<I,D> SaveToFile<I> for EventIter<I,D> where I: Iterator<Item = (D::TimeType,D::ValueType)>, D : Detector {
    fn save_to_file(self, name : &str) -> Result<(),Error> {
        let cd = env::current_dir().unwrap_or_else(|e|log_then_panic_t(format!("Cannot obtain current directory : {e}")));
        let mut file = File::create(cd.join(name)).unwrap_or_else(|e|log_then_panic_t(format!("Cannot create {name} file : {e}")));
        for event in self {
            writeln!(&mut file,"{event}").unwrap_or_else(|e|log_then_panic_t(format!("Cannot write to {name} file : {e}")))
        }
        Ok(())
    }
}
