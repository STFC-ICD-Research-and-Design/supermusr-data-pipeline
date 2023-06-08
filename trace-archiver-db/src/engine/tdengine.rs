use std::fmt::Write;
use flatbuffers::ForwardsUOffset;
use itertools::Itertools;
use rayon::prelude::*;
use rayon::iter;
use taos::taos_query::common::views::IntView;
use taos::taos_query::common::views::TimestampView;
//use crate::tmq::Offset;
//use taos_sys::tmq::Offset;
use taos::{Taos, ResultSet, Stmt, Value, AsyncQueryable, Bindable, ColumnView};
use taos::*;
use std::time::{Instant, Duration};

//use std::thread;

use anyhow::Result;
use async_trait::async_trait;
use common::{Channel, Intensity};
use streaming_types::dat1_digitizer_analog_trace_v1_generated::{DigitizerAnalogTraceMessage, ChannelTrace};

use dotenv;

use super::{framedata::FrameData, test_channels, TimeSeriesEngine};

const MAX_SQL_STRING_LENGTH : usize = 1048576;
 
//  Modify the FrameData struct to add tdengine functionality
impl FrameData {
    /// Appends the table name to the given string
    /// #Arguments
    /// *string - A mutable reference to the string to write to
    /// *channel_number - The channel number of the relevant table
    fn table_name(&self, string : &mut String, channel_number : Channel) -> () {
        write!(string,"d{0}c{1}",self.digitizer_id, channel_number).unwrap();
    }
    /// Appends the frame and channel tags to the given string
    /// #Arguments
    /// *string - A mutable reference to the string to write to
    /// *channel_number - The channel number of the relevant table
    fn sql_tags(&self, string : &mut String, channel_number : Channel) -> () {
        write!(string,"({0}, {1}, {2})",self.digitizer_id, channel_number,self.frame_number).unwrap();
    }
    /// Appends an sql expression to enter the trace data
    /// #Arguments
    /// *string - A mutable to the string to write to
    /// *database - The database to use
    /// *channel - The ChannelTrace structure to enter
    /// Returns
    /// The string that is passed to the method originally in a Result structure, or an error
    fn create_channel_sql_string(&self, mut string : String, database : &str, channel : ChannelTrace) -> Result<String> {
        let channel_number : Channel = channel.channel();
        write!(string, " {database}.")?;
        self.table_name(&mut string,channel_number);
        write!(string, " USING {database}.template TAGS ")?;
        self.sql_tags(&mut string,channel_number);
        write!(string, " VALUES")?;
        for (i,v) in channel.voltage().iter().flatten().enumerate() {
            write!(string, " ({0}, {1})",self.calc_measurement_time(i).timestamp_nanos(),v)?;
        };
        Ok(string)
    }
}





pub struct TDEngine {
    url : String,
    database : String,
    user : String,
    password : String,
    client : Taos,
    sql : Vec<String>,
    stmt : Stmt,
    frame_data : FrameData,
}

impl TDEngine {
    pub async fn new() -> Self {
        let url = format!("{0}:{1}",
            dotenv::var("TDENGINE_URL").unwrap_or_else(|e|panic!("TDEngine URL not found in .env: {}",e)),
            dotenv::var("TDENGINE_PORT").unwrap_or_else(|e|panic!("TDEngine PORT not found in .env: {}",e))
        );
        let database = dotenv::var("TDENGINE_DATABASE").unwrap_or_else(|e|panic!("TDEngine Database not found in .env: {}",e));
        let user = dotenv::var("TDENGINE_USER").unwrap_or_else(|e|panic!("TDEngine User not found in .env: {}",e));
        let password = dotenv::var("TDENGINE_PASSWORD").unwrap_or_else(|e|panic!("TDEngine Password not found in .env: {}",e));
        let client = TaosBuilder::from_dsn(&url).unwrap().build().await.unwrap();
        let stmt = Stmt::init(&client).unwrap();
        TDEngine {
            url, database, user, password,
            client,
            sql : Vec::<String>::default(),
            stmt,
            frame_data: FrameData::default(),
        }
    }

    pub async fn reset_database(&self) -> Result<()> {
        self.client.exec(&format!("DROP DATABASE IF EXISTS {}",self.database)).await.unwrap();
        self.client.exec(&format!("CREATE DATABASE IF NOT EXISTS {} PRECISION 'ns'",self.database)).await.unwrap();
        self.client.exec(&format!("CREATE STABLE IF NOT EXISTS {}.template (ts TIMESTAMP, intensity SMALLINT UNSIGNED) TAGS (detector_id TINYINT UNSIGNED, channel_number INT UNSIGNED, frame_number INT UNSIGNED)",self.database)).await.unwrap();
        self.client.use_database(&self.database).await.unwrap();
        Ok(())
    }

    //  Use Connection To Query Data
    pub async fn query_data(&self, query : &str) -> ResultSet {
        //  Obtain the tsengine client
        self.client.query(query).await.unwrap()
    }



    fn build_statment(&mut self, message: &DigitizerAnalogTraceMessage) -> Result<()> {
        let frame_timestamp_ns = self.frame_data.timestamp.timestamp_nanos();
        let sample_time_ns = self.frame_data.sample_time.num_nanoseconds().unwrap();
        let num_samples = message.channels().unwrap().iter().next().unwrap().voltage().unwrap_or_default().iter().len();
        let sample_timestamps = TimestampView::from_nanos(
            (0_i64..num_samples as i64)
                .map(|i|frame_timestamp_ns + sample_time_ns * i)
                .collect());

        self.stmt.prepare("INSERT INTO ? USING template TAGS(?) VALUES(?, ?)")?;
        let tags = [
            //Value::UTinyInt(self.frame_data.digitizer_id),
            //Value::Null(taos::Ty::UInt),
            Value::UInt(self.frame_data.frame_number),
        ];
        self.stmt.set_tags(&tags).unwrap();
        //let mut total_time = Duration::default();
        for channel in message.channels().unwrap() {
            let mut tbname = "".to_owned();//format!("{0}.", self.database);
            self.frame_data.table_name(&mut tbname, channel.channel());
            self.stmt.set_tbname(tbname).unwrap();
            //tags[1] = Value::UInt(channel.channel());
            let values = channel.voltage()
                .unwrap()
                .iter()
                .collect::<Vec<u16>>();
            if let Err(e) = self.stmt.bind(&[
                ColumnView::Timestamp(sample_timestamps.clone()),
                ColumnView::from_unsigned_small_ints(values),
            ]) {
                return Err(e.into())
            }
            self.stmt.add_batch()?;
        }
        //println!("{:?}", total_time);
        Ok(())
    }

    fn build_strings(&mut self, message: &DigitizerAnalogTraceMessage) -> Result<()> {
        self.sql.clear();
        message.channels().unwrap().iter().for_each(|channel| {
            self.sql.push(self.frame_data.create_channel_sql_string("INSERT INTO".to_owned(),&self.database,channel).unwrap());
        });
        Ok(())
    }

}

#[async_trait]
impl TimeSeriesEngine for TDEngine {
    /// Takes a reference to a ``DigitizerAnalogTraceMessage`` instance and extracts the relevant data from it.
    /// The user should then call ``post_message`` to send the data to the tdengine server.
    /// Calling this method erases all data extracted from previous calls the ``process_message``.
    /// #Arguments
    /// *message - The ``DigitizerAnalogTraceMessage`` instance from which the data is extracted
    /// #Returns
    /// An emtpy result or an error arrising a malformed ``DigitizerAnalogTraceMessage`` parameter.
    async fn process_message(&mut self, message: &DigitizerAnalogTraceMessage) -> Result<()> {
        // Obtain message data, and error check
        self.frame_data.init(message)?;
        // Obtain the channel data, and error check
        test_channels(message)?;

        self.build_statment(message)?;
        //self.build_strings(message)?;
        Ok(())
    }


    /// Sends data extracted from a previous call to ``process_message`` to the influxdb server.
    /// #Returns
    /// A string result, or an error arrising from the influxdb queries.
    async fn post_message(&mut self) -> Result<String> {
        
        if !self.sql.is_empty() {
            let result : Result<usize,_> = self.client.exec_many(&self.sql).await;
            if let Err(e) = result {
                panic!("SQL {} Caused Error {}",self.sql.len(),e);
            }
            Ok(format!("SQL statement affected {0} rows",
                result.unwrap()))
        }
        else{
            Ok(format!("SQL statement affected {0} rows",
                self.stmt.execute().unwrap()))
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{simulator, TDEngine, engine::TimeSeriesEngine};
    use taos::*;
    use flatbuffers::FlatBufferBuilder;
    use streaming_types::dat1_digitizer_analog_trace_v1_generated::{DigitizerAnalogTraceMessage,root_as_digitizer_analog_trace_message};
    
    #[tokio::test]
    async fn test_create_db() {
        let engine = TDEngine::new().await;
        assert!(engine.reset_database().await.is_ok());
    }

    async fn assert_count_eq(res : &mut ResultSet, val : usize) -> Result<(),Error> {
        assert_eq!(res.fields().len(),1);
        res.fields().into_iter().for_each(|f|assert_eq!(f.name(), "count(*)"));
        
        let mut num_rows: i32 = 0;
        
        while let Some(row) = res.rows().try_next().await? {
            num_rows += 1;
            assert_eq!(row.len(),1);
            row.for_each(|(n, v)|
                assert_eq!((n, v.to_value()), ("count(*)", Value::BigInt(val as i64))))
        }
        assert_eq!(num_rows,1);
        Ok(())
    }

    #[tokio::test]
    async fn test_create_insert_and_count() {
        let mut fbb: FlatBufferBuilder = FlatBufferBuilder::new();

        let digitizer_id = 0..=24;
        let frame_number = 0..=5;
        let measurements_per_frame : usize = 16;
        let num_channels : usize = 4;
        
        simulator::create_partly_random_message_with_now(&mut fbb, frame_number, digitizer_id, measurements_per_frame, num_channels).unwrap();
        let msg : DigitizerAnalogTraceMessage = root_as_digitizer_analog_trace_message(fbb.finished_data()).unwrap();

        let mut engine = TDEngine::new().await;
        engine.reset_database().await.unwrap();

        let mut res: ResultSet = engine.query_data("SELECT COUNT(*) FROM TraceLogs.template").await;
        assert_count_eq(&mut res,0).await.unwrap();

        engine.process_message(&msg).await.unwrap();
        let string = engine.post_message().await.unwrap();
        assert_eq!(string, format!("SQL statement affected {0} rows", measurements_per_frame*num_channels));

        let mut res = engine.query_data("SELECT COUNT(*) FROM TraceLogs.template").await;
        assert_count_eq(&mut res, measurements_per_frame*num_channels).await.unwrap();
    }
}
