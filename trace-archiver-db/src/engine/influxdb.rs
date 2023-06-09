use std::default;

use anyhow::Result;
use async_trait::async_trait;
use common::{Channel, Intensity, FrameNumber, DigitizerId};
use chrono::{DateTime, Utc};
use streaming_types::dat1_digitizer_analog_trace_v1_generated::DigitizerAnalogTraceMessage;
use influxdb::{ReadQuery, WriteQuery, Client, InfluxDbWriteable};

use super::{framedata::FrameData, TimeSeriesEngine};



//  Modify the FrameData struct to add influxdb functionality
impl FrameData {
    /// Create an influxdb Measurement instance with the given channel number and voltage
    /// #Arguments
    /// * `channel_number` - The index of the channel
    /// * `index` - The index of the measurement in the frame
    /// * `voltage` - The voltage of the measurement
    /// #Returns
    /// A Measurement instance
    fn make_measurement(&self, channel_number : Channel, index : usize, voltage : Intensity) -> Measurement {
        Measurement {
            time: self.calc_measurement_time(index),
            digitizer_id: self.digitizer_id,
            frame_number: self.frame_number,
            channel: channel_number,
            intensity: voltage as i32,
        }
    }
}


/// A structure representing an influxdb measurement, as it derives InfluxDbWriteable
/// it implements the WriteQuery function to send the measurement to the influxdb server.
/// #Fields
/// *time - a DateTime representing the measurement time.
/// *digitizer_id - the id of the digitizer marked as a tag.
/// *frame_number - the number of the frame marked as a tag.
/// *channel - the index of the channel marked as a tag.
/// *intesity - the intensity of the measurement, the sole field of the measurement.
#[derive(InfluxDbWriteable, Default)]
struct Measurement {
    time: DateTime<Utc>,
    #[influxdb(tag)]digitizer_id : DigitizerId,
    #[influxdb(tag)]frame_number : FrameNumber,
    #[influxdb(tag)]channel : Channel,
    #[doc = "Using type `Intensity` causes an error"]
    intensity: i32,//using type Intensity causes an error
}

/// A structure representing the influxdb engine.
/// #Fields
/// *client - a DateTime representing the measurement time.
/// *frame_data - the id of the digitizer marked as a tag.
/// *measurements - a vector of consisting of the measurements to write to the influxdb server.
pub(crate) struct InfluxDBEngine {
    url : String,
    database : String,
    client : Client,
    frame_data : FrameData,
    measurements : Vec<WriteQuery>,
}

impl InfluxDBEngine {
    /// Creates a new instance of InfluxDBEngine
    /// #Returns
    /// An instance connected to "http://localhost:8086" and database "TraceLogs".
    /// The token used to authenticate with the influxdb server is currently hardcoded.
    pub async fn new() -> Self {
        let url = dotenv::var("INFLUXDB_URL").unwrap_or_else(|e|panic!("INFLUXDB_URL not found in .env: {}",e));
        let database = dotenv::var("INFLUXDB_DATABASE").unwrap_or_else(|e|panic!("INFLUXDB_DATABASE not found in .env: {}",e));
        let token = dotenv::var("INFLUXDB_TOKEN").unwrap_or_else(|e|panic!("INFLUXDB_TOKEN not found in .env: {}",e));
        InfluxDBEngine {
            url : url.clone(),
            database : database.clone(),
            client: Client::new(url,database).with_token(token),/*with_auth("admin", "password"),*/
            frame_data: FrameData::default(),
            measurements: Vec::<WriteQuery>::default(),
        }
    }

    /// Clears all data from database "TraceLogs" and resets it.
    /// #Returns
    /// An emtpy result or an error arrising from the influxdb queries.
    pub async fn reset_database(&self) -> Result<()> {
        self.client.query(ReadQuery::new(format!("DROP DATABASE {}",self.database))).await?;
        self.client.query(ReadQuery::new(format!("CREATE DATABASE {}",self.database))).await?;
        Ok(())
    }
}

#[async_trait]
impl TimeSeriesEngine for InfluxDBEngine {
    /// Takes a reference to a ``DigitizerAnalogTraceMessage`` instance and extracts the relevant data from it.
    /// The user should then call ``post_message`` to send the data to the influxdb server.
    /// Calling this method erases all data extracted from previous calls the ``process_message``.
    /// #Arguments
    /// *message - The ``DigitizerAnalogTraceMessage`` instance from which the data is extracted
    /// #Returns
    /// An emtpy result or an error arrising a malformed ``DigitizerAnalogTraceMessage`` parameter.
    async fn process_message(&mut self, message: &DigitizerAnalogTraceMessage) -> Result<()> {
        self.measurements.clear();
        // Obtain message data, and error check
        self.frame_data.init(message)?;
        //test_channels(message,8).unwrap();  //  TODO influxdb is used then this should be implemented properly

        for channel in message.channels().unwrap() {
            for (i,v) in channel.voltage().iter().flatten().enumerate() {
                self.measurements.push(
                    self.frame_data.make_measurement(channel.channel(),i,v).into_query("template")
                );
            }
        }
        Ok(())
    }
    /// Sends data extracted from a previous call to ``process_message`` to the influxdb server.
    /// #Returns
    /// A string result, or an error arrising from the influxdb queries.
    async fn post_message(&mut self) -> Result<String> {
        self.client.query(&self.measurements).await?;
        Ok("".to_owned())
    }
}



#[cfg(test)]
mod test {
    use influxdb::ReadQuery;

    use super::*;

    #[tokio::test]
    async fn test_create() {
        let influx_db: InfluxDBEngine = InfluxDBEngine::new().await;
        assert!(influx_db.client.ping().await.is_ok());
    }
    #[tokio::test]
    async fn test_database_name() {
        let influx_db: InfluxDBEngine = InfluxDBEngine::new().await;
        assert_eq!(influx_db.client.database_name(),influx_db.database);
    }

    #[tokio::test]
    async fn test_insert() {
        let influx_db: InfluxDBEngine = InfluxDBEngine::new().await;
        influx_db.reset_database().await.unwrap();
        let write_result: std::result::Result<String, influxdb::Error> = influx_db.client.query(Measurement::default().into_query("template")).await;
        assert!(write_result.is_ok());
    }
    #[tokio::test]
    async fn test_query() {
        let influx_db: InfluxDBEngine = InfluxDBEngine::new().await;
        influx_db.reset_database().await.unwrap();
        let query = ReadQuery::new(format!("SELECT * from {}",influx_db.database));
        let read_result = influx_db.client.query(query).await;
        assert!(read_result.is_ok());
    }
    #[tokio::test]
    async fn test_insert_and_query() {
        let influx_db: InfluxDBEngine = InfluxDBEngine::new().await;
        influx_db.reset_database().await.unwrap();
        let write_result = influx_db.client.query(Measurement{
            time:DateTime::<Utc>::from_utc(chrono::NaiveDate::from_ymd_opt(2000,1,1).unwrap().and_hms_nano_opt(2, 0, 0,10_000).unwrap(),Utc),
            digitizer_id:4,
            frame_number:0,
            channel:6,
            intensity:23,
        }.into_query("template")).await;
        let query = ReadQuery::new("SELECT * from template WHERE time >= '2000-01-01 02:00:00'");
        assert!(write_result.is_ok());
        let read_result = influx_db.client.query(query).await;
        //assert!(read_result.is_ok());
        println!("{}",read_result.unwrap());
    }
}
