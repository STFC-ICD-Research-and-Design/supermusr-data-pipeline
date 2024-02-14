use std::{fs::create_dir_all, path::Path};

use anyhow::{anyhow, Result};

use chrono::{DateTime, Utc, Duration};
use hdf5::{
    types::VarLenUnicode, Dataset, File, Group, SimpleExtents
};
use supermusr_common::{Channel, Time};
use tracing::debug;

use crate::nexus::{hdf5_writer::{add_new_group_to, set_attribute_list_to, set_group_nx_class}, nexus_class as NX, run_parameters::DATETIME_FORMAT};
use ndarray::s;
const TIMESTAMP_FORMAT: &str = "%Y-%m-%dT%H:%M:%S%.f%z";

use super::{run_parameters::RunParameters, GenericEventMessage};

#[derive(Debug)]
pub(crate) struct Run {
    pub(crate) run_parameters: RunParameters,
    file: File,
    entry: Group,
    
    idf_version : Dataset,
    definition : Dataset,
    run_number : Dataset,
    experiment_identifier : Dataset,

    start_time : Dataset,
    end_time : Dataset,
    name : Dataset,
    title : Dataset,

    instrument: Group,
    instrument_name : Dataset,

    detector: Group,

    source: Group,
    source_name : Dataset,
    source_type : Dataset,
    source_probe : Dataset,
    
    periods: Group,
    period_number : Dataset,
    period_type : Dataset,

    lists : EventRun,
}

impl Run {
    pub(crate) fn new(filename: &Path, run_parameters: RunParameters) -> Result<Self> {
        create_dir_all(filename)?;
        let filename = {
            let mut filename = filename.to_owned();
            filename.push(run_parameters.run_name.as_str());
            filename.set_extension("nxs");
            filename
        };
        debug!("File save begin. File: {0}.", filename.display());

        let file = File::create(filename)?;
        set_group_nx_class(&file, NX::ROOT)?;

        set_attribute_list_to(
            &file,
            &[
                ("HDF5_version", "1.14.3"), // Can this be taken directly from the nix package?
                ("NeXus_version", ""),      // Where does this come from?
                ("file_name", &file.filename()), //  This should be absolutized at some point
                ("file_time", Utc::now().to_string().as_str()), //  This should be formatted, the nanoseconds are overkill.
            ],
        )?;

        let entry = add_new_group_to(&file, "raw_data_1", NX::ENTRY)?;

        let idf_version = entry.new_dataset::<u32>().create("IDF_version")?;
        let definition = entry.new_dataset::<VarLenUnicode>()
            .create("definition")?;
        let run_number = entry.new_dataset::<u32>().create("run_number")?;
        let experiment_identifier = entry.new_dataset::<VarLenUnicode>()
            .create("experiment_identifier")?;

        let start_time = entry.new_dataset::<VarLenUnicode>()
            .create("start_time")?;
        let end_time = entry.new_dataset::<VarLenUnicode>()
            .create("end_time")?;

        let name = entry.new_dataset::<VarLenUnicode>()
            .create("name")?;
        let title = entry.new_dataset::<VarLenUnicode>()
            .create("title")?;
        
        let instrument = add_new_group_to(&entry, "instrument", NX::INSTRUMENT)?;
        let instrument_name = instrument.new_dataset::<VarLenUnicode>()
            .create("name")?;

        let periods = add_new_group_to(&entry, "periods", NX::PERIOD)?;
        let period_number = periods.new_dataset::<u32>()
            .create("number")?;
        let period_type = periods.new_dataset::<u32>()
            .shape(SimpleExtents::resizable(vec![0]))
            .chunk(vec![32])
            .create("type")?;

        let source = add_new_group_to(&instrument, "source", NX::SOURCE)?;
        let source_name = source.new_dataset::<VarLenUnicode>()
            .create("name")?;
        let source_type = source.new_dataset::<VarLenUnicode>()
            .create("type")?;
        let source_probe = source.new_dataset::<VarLenUnicode>()
            .create("probe")?;

        let detector = add_new_group_to(&instrument, "detector", NX::DETECTOR)?;

        let lists = EventRun::new(&entry)?;
        
        Ok(Self {
            run_parameters,
            file,
            detector,
            entry,
            idf_version,
            start_time,
            end_time,
            name,
            title,
            instrument,
            instrument_name,
            periods,
            period_number,
            period_type,
            lists,
            source,
            source_name,
            source_type,
            source_probe,
            definition,
            run_number,
            experiment_identifier,
        })
    }

    pub(crate) fn init(&mut self) -> Result<()> {
        self.idf_version.write_scalar(&2).unwrap();
        self.definition.write_scalar(&"muonTD".parse::<VarLenUnicode>()?).unwrap();
        self.run_number.write_scalar(&self.run_parameters.run_number).unwrap();
        self.experiment_identifier.write_scalar(&"".parse::<VarLenUnicode>()?).unwrap();
        
        let start_time = DateTime::<Utc>::from_timestamp_millis(self.run_parameters.collect_from as i64)
            .ok_or(anyhow!("Cannot create start_time from {0}",self.run_parameters.collect_from))?
            .format(DATETIME_FORMAT)
            .to_string();
        self.start_time.write_scalar(&start_time.parse::<VarLenUnicode>()?).unwrap();

        self.name.write_scalar(&self.run_parameters.run_name.parse::<VarLenUnicode>()?).unwrap();
        self.title.write_scalar(&"".parse::<VarLenUnicode>()?).unwrap();

        self.instrument_name.write_scalar(&self.run_parameters.instrument_name.parse::<VarLenUnicode>()?).unwrap();

        self.period_number.write_scalar(&self.run_parameters.num_periods).unwrap();
        self.period_type.resize(self.run_parameters.num_periods as usize).unwrap();
        self.period_type.write_raw(&vec![1; self.run_parameters.num_periods as usize]).unwrap();

        self.source_name.write_scalar(&"MuSR".parse::<VarLenUnicode>()?).unwrap();
        self.source_type.write_scalar(&"".parse::<VarLenUnicode>()?).unwrap();
        self.source_probe.write_scalar(&"".parse::<VarLenUnicode>()?).unwrap();
        Ok(())
    }

    pub(crate) fn finish(&mut self, message : &GenericEventMessage) -> Result<()> {
        let end_ms = if let Some(run_stop_parameters) = &self.run_parameters.run_stop_parameters {
            run_stop_parameters.collect_until as i64
        } else {
            let ns = message.time.map(|time|time.iter().last())
                .flatten()
                .ok_or(anyhow!("Event time missing."))?;

            message.timestamp.timestamp_millis() + ns.div_ceil(1_000_000) as i64
        };
        let end_time = DateTime::<Utc>::from_timestamp_millis(end_ms)
            .ok_or(anyhow!("Cannot create end_time from {end_ms}"))?
            .format(DATETIME_FORMAT)
            .to_string();
        self.end_time.write_scalar(&end_time.parse::<VarLenUnicode>()?)?;
        Ok(())
    }

    pub(crate) fn push_message(&mut self, message : &GenericEventMessage) -> Result<()> {
        self.lists.push_message(message)?;
        self.finish(message)?;
        Ok(())
    }
    pub(crate) fn close(self) -> Result<()> {
        self.file.close()?;
        Ok(())
    }
}


#[derive(Debug)]
struct EventRun {
    offset: Option<DateTime<Utc>>,

    num_messages : usize,
    num_events : usize,

    detector: Group,
    //  Frames
    event_index : Dataset,
    event_time_zero : Dataset,
    //  Events
    event_id : Dataset,
    pulse_height : Dataset,
    event_time_offset : Dataset,
}

impl EventRun {
    pub(crate) fn new(parent : &Group) -> Result<Self> {
        let detector = add_new_group_to(parent, "detector_1", NX::EVENT_DATA)?;
        
        let pulse_height = detector.new_dataset::<f64>()
            .shape(SimpleExtents::resizable(vec![0]))
            .chunk(vec![1024])
            .create("pulse_height")?;
        let event_id = detector.new_dataset::<Channel>()
            .shape(SimpleExtents::resizable(vec![0]))
            .chunk(vec![1024])
            .create("event_id")?;
        let event_time_offset = detector.new_dataset::<Time>()
            .shape(SimpleExtents::resizable(vec![0]))
            .chunk(vec![1024])
            .create("event_time_offset")?;
        set_attribute_list_to(&event_time_offset, &[("units", "ns")])?;

        let event_index = detector.new_dataset::<u32>()
            .shape(SimpleExtents::resizable(vec![0]))
            .chunk(vec![64])
            .create("event_index")?;
        let event_time_zero = detector.new_dataset::<u64>()
            .shape(SimpleExtents::resizable(vec![0]))
            .chunk(vec![64])
            .create("event_time_zero")?;
        set_attribute_list_to(&event_time_zero, &[("units", "ns")])?;
        
        Ok(Self {
            offset: None,
            num_messages: 0,
            num_events: 0,
            detector,
            event_id,
            event_index,
            pulse_height,
            event_time_offset,
            event_time_zero,
        })
    }

    pub(crate) fn push_message(&mut self, message : &GenericEventMessage) -> Result<()> {
        self.event_index.resize(self.num_messages + 1).unwrap();
        self.event_index.write_slice(&[self.num_events], s![self.num_messages..(self.num_messages + 1)]).unwrap();
        
        let timestamp = Into::<DateTime<Utc>>::into(
            *message.metadata
                .timestamp()
                .ok_or(anyhow!("Message timestamp missing."))?
        );

        self.event_time_zero.resize(self.num_messages + 1).unwrap();
        let time_zero = {
            if let Some(offset) = self.offset {
                debug!("Offset found");
                timestamp - offset
            } else {
                set_attribute_list_to(&self.event_time_zero, &[(
                    "offset",
                    &timestamp
                        .format(TIMESTAMP_FORMAT)
                        .to_string(),
                )])?;
                self.offset = Some(timestamp);
                debug!("New offset set");
                Duration::zero()
            }
        }.num_nanoseconds()
        .ok_or(anyhow!("event_time_zero cannot be calculated."))? as u64;
        self.event_time_zero.write_slice(&[time_zero], s![self.num_messages..(self.num_messages + 1)])?;

        let num_new_events = message.channel.unwrap_or_default().len();
        let total_events = self.num_events + num_new_events;

        self.pulse_height.resize(total_events)?;
        self.pulse_height.write_slice(&message.voltage.unwrap_or_default().iter().collect::<Vec<_>>(), s![self.num_events..total_events])?;

        self.event_time_offset.resize(total_events)?;
        self.event_time_offset.write_slice(&message.time.unwrap_or_default().iter().collect::<Vec<_>>(), s![self.num_events..total_events])?;

        self.event_id.resize(total_events)?;
        self.event_id.write_slice(&message.channel.unwrap_or_default().iter().collect::<Vec<_>>(), s![self.num_events..total_events])?;

        self.num_events = total_events;
        self.num_messages += 1;
        Ok(())
    }
}