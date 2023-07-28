use std::iter::{repeat, once};
use async_trait::async_trait;
use anyhow::Result;

use taos::*;

use streaming_types::dat1_digitizer_analog_trace_v1_generated::{DigitizerAnalogTraceMessage, ChannelTrace};
use common::{Channel, Intensity};

use crate::utils::log_then_panic_t;

use super::error::{self, SQLError};
use super::error::{ChannelError, TraceMessageError, TDEngineError, StatementError};
use super::tdengine_views as views;

use super::{tdengine_login::TDEngineLogin, error_reporter::TDEngineErrorReporter, framedata::FrameData, TimeSeriesEngine};

pub struct TDEngine {
    login : TDEngineLogin,
    client : Taos,
    stmt : Stmt,
    frame_stmt : Stmt,
    error : TDEngineErrorReporter,
    frame_data : FrameData,
}

impl TDEngine {
    pub async fn from_env() -> Self { Self::from_optional(None,None,None,None,None).await }
    pub async fn from_optional(url : Option<String>, port : Option<u32>, username : Option<String>, password : Option<String>, database : Option<String>) -> Self {
        let login = TDEngineLogin::from_optional(url, port, username, password, database);
        log::debug!("Creating TaosBuilder with login {login:?}");
        let client = TaosBuilder::from_dsn(login.get_url())
            .unwrap_or_else(|e|log_then_panic_t(format!("Unable to create TaosBuilder with dsn: {login:?}. Error: {e}")))
            .build().await
            .unwrap_or_else(|e|log_then_panic_t(format!("Unable to build Taos with dsn: {login:?}. Error: {e}")));
        let stmt = Stmt::init(&client)
            .unwrap_or_else(|e|log_then_panic_t(format!("Unable to init Taos Statement : {e}")));
        let frame_stmt = Stmt::init(&client)
            .unwrap_or_else(|e|log_then_panic_t(format!("Unable to init Taos Statement : {e}")));
        TDEngine {login, client, stmt, frame_stmt,
            error: TDEngineErrorReporter::default(),
            frame_data: FrameData::default(),
        }
    }

    pub async fn delete_database(&self) -> Result<()> {
        self.client.exec(&format!("DROP DATABASE IF EXISTS {}",self.login.get_database())).await?;
        Ok(())
    }

    pub async fn create_database(&self) -> Result<()> {
        self.client.exec(&format!("CREATE DATABASE IF NOT EXISTS {} PRECISION 'ns'",self.login.get_database())).await?;
        self.client.use_database(self.login.get_database()).await?;
        Ok(())
    }
    async fn create_supertable(&self) -> Result<(),error::Error> {
        let metrics_string = format!("ts TIMESTAMP, frametime TIMESTAMP{0}",
            (0..self.frame_data.num_channels)
                .map(|ch|format!(", c{ch} SMALLINT UNSIGNED"))
                .fold(String::new(),|a,b|a + &b));
        let string = format!("CREATE STABLE IF NOT EXISTS template ({metrics_string}) TAGS (digitizer_id TINYINT UNSIGNED)");
        self.client.exec(&string).await.map_err(|e|TDEngineError::SQL(SQLError::CreateTemplateTable, string.clone(), e))?;

        let frame_metrics_string = format!("frame_ts TIMESTAMP, sample_count INT UNSIGNED, sampling_rate INT UNSIGNED, frame_number INT UNSIGNED, has_error BOOL{0}",
            (0..self.frame_data.num_channels)
                .map(|ch|format!(", cid{ch} INT UNSIGNED"))
                .fold(String::new(),|a,b|a + &b)
        );
        let string = format!("CREATE STABLE IF NOT EXISTS frame_template ({frame_metrics_string}) TAGS (digitizer_id TINYINT UNSIGNED)");
        self.client.exec(&string).await.map_err(|e|TDEngineError::SQL(SQLError::CreateTemplateTable, string.clone(), e))?;
        Ok(())
    }/*
    async fn create_supertable(&self) -> Result<(),error::Error> {
        let metrics_string = format!("ts TIMESTAMP{0}",
            (0..self.frame_data.num_channels)
                .map(|ch|format!(", channel{ch} SMALLINT UNSIGNED"))
                .fold(String::new(),|a,b|a + &b));
        let tags_string = format!("frame_start_time TIMESTAMP, frame_sampling_rate INT UNSIGNED, frame_number INT UNSIGNED{0}, has_error BOOL",
            (0..self.frame_data.num_channels)
                .map(|ch|format!(", channel_id{ch} INT UNSIGNED"))
                .fold(String::new(),|a,b|a + &b)
        );
        let string = format!("CREATE STABLE IF NOT EXISTS template ({metrics_string}) TAGS ({tags_string})");
        self.client.exec(&string).await.map_err(|e|TDEngineError::SQL(SQLError::CreateTemplateTable, string.clone(), e))?;
        Ok(())
    }*/
    pub async fn init_with_channel_count(&mut self, num_channels : usize) -> Result<(), error::Error> {
        self.frame_data.set_channel_count(num_channels);
        self.create_supertable().await?;

        
        //let stmt_sql = format!("INSERT INTO ? USING template TAGS (?, ?, ?{0}, ?) VALUES (?{0})", ", ?".repeat(num_channels));
        let stmt_sql = format!("INSERT INTO ? USING template TAGS (?) VALUES (?, ?{0})", ", ?".repeat(num_channels));
        self.stmt.prepare(&stmt_sql).map_err(|e|TDEngineError::Stmt(StatementError::Prepare, e))?;

        let frame_stmt_sql = format!("INSERT INTO ? USING frame_template TAGS (?) VALUES (?, ?, ?, ?, ?{0})", ", ?".repeat(num_channels));
        self.frame_stmt.prepare(&frame_stmt_sql).map_err(|e|TDEngineError::Stmt(StatementError::Prepare, e))?;
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
    async fn process_message(&mut self, message: &DigitizerAnalogTraceMessage) -> Result<(),error::Error> {
        // Obtain the channel data, and error check
        self.error.test_metadata(&message);

        // Obtain message data, and error check
        self.frame_data.init(message)?;

        // Obtain the channel data, and error check
        self.error.test_channels(&self.frame_data, &message.channels().unwrap());

        let table_name = self.frame_data.get_table_name();
        let frame_table_name = self.frame_data.get_frame_table_name();
        let frame_column_views = views::create_frame_column_views(&self.frame_data, &self.error, &message.channels().unwrap());
        let column_views = views::create_column_views(&self.frame_data, &message.channels().unwrap());
        let tags = [Value::UTinyInt(self.frame_data.digitizer_id)];

        //  Initialise Statement
        self.stmt.set_tbname(table_name).map_err(   |e|TDEngineError::Stmt(StatementError::SetTableName, e))?;
        self.stmt.set_tags(&tags).map_err(          |e|TDEngineError::Stmt(StatementError::SetTags, e))?;
        self.stmt.bind(&column_views).map_err(      |e|TDEngineError::Stmt(StatementError::Bind, e))?;
        self.stmt.add_batch().map_err(              |e|TDEngineError::Stmt(StatementError::AddBatch, e))?;
        
        self.frame_stmt.set_tbname(frame_table_name).map_err(   |e|TDEngineError::Stmt(StatementError::SetTableName, e))?;
        self.frame_stmt.set_tags(&tags).map_err(                |e|TDEngineError::Stmt(StatementError::SetTags, e))?;
        self.frame_stmt.bind(&frame_column_views).map_err(      |e|TDEngineError::Stmt(StatementError::Bind, e))?;
        self.frame_stmt.add_batch().map_err(                    |e|TDEngineError::Stmt(StatementError::AddBatch, e))?;
        Ok(())
    }


    /// Sends data extracted from a previous call to ``process_message`` to the tdengine server.
    /// #Returns
    /// The number of rows affected by the post or an error
    async fn post_message(&mut self) -> Result<usize,error::Error> {
        self.stmt.execute().map_err(|e|error::Error::TDEngine(TDEngineError::Stmt(StatementError::Execute, e))).unwrap();
        self.frame_stmt.execute().map_err(|e|error::Error::TDEngine(TDEngineError::Stmt(StatementError::Execute, e))).unwrap();
        Ok(0)
    }
}










// Note these tests should be run in single-threaded mode
// cargo test -- --test-threads 1 --show-output
#[cfg(test)]
mod test {
    use anyhow::{anyhow, Result};
    use chrono::{DateTime, Utc};
    use common::{FrameNumber, Intensity, Channel};
    use rand::rngs::ThreadRng;
    use serde::Deserialize;

    use taos::*;
    use flatbuffers::FlatBufferBuilder;

    use streaming_types::dat1_digitizer_analog_trace_v1_generated::{DigitizerAnalogTraceMessage,root_as_digitizer_analog_trace_message};

    use trace_simulator::{self, Malform, MalformType};
    use super::{ TDEngine, TimeSeriesEngine, error::TraceMessageError};

    #[derive(Debug, serde::Deserialize)]
    struct SingleI32QueryRecord(i32);
    #[derive(Debug, serde::Deserialize)]
    struct TwoChannelQueryRecord {
        ts : DateTime<Utc>,
        channel0 : Intensity, channel1 : Intensity,
        frame_number : FrameNumber,
        channel_id0 : Channel, channel_id1 : Channel,
        has_error : bool,
    }
    #[derive(Debug, serde::Deserialize)]
    struct FourChannelQueryRecord {
        ts : DateTime<Utc>,
        channel0 : Intensity, channel1 : Intensity, channel2 : Intensity, channel3 : Intensity,
        frame_number : FrameNumber,
        channel_id0 : Channel, channel_id1 : Channel, channel_id2 : Channel, channel_id3 : Channel,
        has_error : bool,
    }

    impl TDEngine {
        async fn delete_test_database(&self) -> Result<()> {
            self.client.exec("DROP DATABASE IF EXISTS test_database").await.unwrap();
            Ok(())
        }
        async fn create_test_database(&self) -> Result<()> {
            self.client.exec("CREATE DATABASE IF NOT EXISTS test_database PRECISION 'ns'").await.unwrap();
            self.client.use_database("test_database").await.unwrap();
            Ok(())
        }
        
        ///
        async fn single_field_query(&self, sql : &str) -> Vec<SingleI32QueryRecord> {
            self.client.query(&sql).await.unwrap().deserialize().try_collect().await.unwrap() }
        
        ///
        async fn two_channel_query(&self, sql : &str) -> Vec<TwoChannelQueryRecord> {
            self.client.query(&sql).await.unwrap().deserialize().try_collect().await.unwrap() }
        
        ///
        async fn four_channel_query(&self, sql : &str) -> Vec<FourChannelQueryRecord> {
            self.client.query(&sql).await.unwrap().deserialize().try_collect().await.unwrap() }

        ///
        async fn get_count(&self, table : &str) -> Result<i32> {
            let sql = format!("SELECT COUNT(*) FROM {table}");
            let count_vec = self.single_field_query(&sql).await;
            if count_vec.is_empty() {
                panic!("{sql} returned empty count")
            } else {
                Ok(count_vec[0].0)
            }
        }
    }

    ///  A module of functions that test assertions
    mod assert {
        use super::*;

        pub(super) struct ChannelParameters {
            pub(super) id : Channel,
            pub(super) samples : usize,
        }
        pub(super) async fn two_channels_correct(engine : &TDEngine, channel : [ChannelParameters;2], measurements_per_frame : usize, has_error : bool) {
            let results = engine.two_channel_query("SELECT * FROM template").await;
            assert_eq!(results.len(),measurements_per_frame);
            for result in &results {
                assert_eq!(result.channel_id0,channel[0].id, "Failed with record {result:?}");
                assert_eq!(result.channel_id1,channel[1].id, "Failed with record {result:?}");
                assert_eq!(result.has_error,has_error, "Failed with record {result:?}");
            }
            
            for (i,result) in results.iter().enumerate() {
                if i >= channel[0].samples { assert_eq!(result.channel0, 0); }
                if i >= channel[1].samples { assert_eq!(result.channel1, 0); }
            }
        }

        pub(super) async fn four_channels_correct(engine : &TDEngine, channel : [ChannelParameters;4], measurements_per_frame : usize, has_error : bool) {
            let results = engine.four_channel_query("SELECT * FROM template").await;
            assert_eq!(results.len(),measurements_per_frame);
            for result in &results {
                assert_eq!(result.channel_id0,channel[0].id, "Failed with record {result:?}");
                assert_eq!(result.channel_id1,channel[1].id, "Failed with record {result:?}");
                assert_eq!(result.channel_id2,channel[2].id, "Failed with record {result:?}");
                assert_eq!(result.channel_id3,channel[3].id, "Failed with record {result:?}");
                assert_eq!(result.has_error,has_error, "Failed with record {result:?}");
            }
            
            for (i,result) in results.iter().enumerate() {
                if i >= channel[0].samples { assert_eq!(result.channel0, 0); }
                if i >= channel[1].samples { assert_eq!(result.channel1, 0); }
                if i >= channel[2].samples { assert_eq!(result.channel2, 0); }
                if i >= channel[3].samples { assert_eq!(result.channel3, 0); }
            }
        }

        pub(super) fn correct_error_reports_filed(engine : &TDEngine, expected_reports : Vec<String>) {
            assert_eq!(expected_reports.len(), engine.error.num_errors());
            for error_result in engine.error.reports_iter() {
                assert!(expected_reports.contains(&error_result));
            }
        }

        pub(super) async fn number_or_rows_with_errors_two_channel(engine : &TDEngine, expected_rows_with_error : usize) {
            let results = engine.two_channel_query("SELECT * FROM template WHERE has_error = True").await;
            assert_eq!(results.len(), expected_rows_with_error);

        }

        pub(super) async fn number_or_rows_with_errors_four_channel(engine : &TDEngine, expected_rows_with_error : usize) {
            let results = engine.four_channel_query("SELECT * FROM template WHERE has_error = True").await;
            assert_eq!(results.len(), expected_rows_with_error);
        }
    }

    // Functions used in every test for setup

    ///Creates a new tdengine instance and initialises it in test mode with the given number of channels
    /// #Arguments
    /// *num_channels - Number of channels
    /// #Returns
    /// A TDEngine instance
    async fn create_test_engine(num_channels : usize) -> TDEngine {
        let mut engine = TDEngine::from_env().await;
        engine.delete_test_database().await.unwrap();
        engine.create_test_database().await.unwrap();
        engine.init_with_channel_count(num_channels).await.unwrap();
        engine
    }

    ///Creates a new DigitizerAnalogTraceMessage instance and initialises with some given parameters and randomizes some others
    /// #Arguments
    /// *fbb - TODO
    /// *num_channels - Number of channels
    /// *malform - TODO
    /// #Returns
    /// A DigitizerAnalogTraceMessage instance
    fn generate_message<'b, 'a : 'b>(fbb : &'a mut FlatBufferBuilder, measurements_per_frame : usize, num_channels : usize, malform : &Malform) -> DigitizerAnalogTraceMessage<'b> {
        let digitizer_id = 0..=24;
        let frame_number = 0..=5;
        
        trace_simulator::create_partly_random_message_with_now(fbb, frame_number, digitizer_id, measurements_per_frame, num_channels, malform).unwrap();
        root_as_digitizer_analog_trace_message(fbb.finished_data()).unwrap()
    }


    #[tokio::test]
    async fn test_create_db() {
        let engine = TDEngine::from_env().await;
        assert!(engine.delete_test_database().await.is_ok());
        assert!(engine.create_test_database().await.is_ok());
    }

    #[tokio::test]
    async fn test_create_tables() {
        let mut engine = TDEngine::from_env().await;
        assert!(engine.delete_test_database().await.is_ok());
        assert!(engine.create_test_database().await.is_ok());
        assert!(engine.init_with_channel_count(8).await.is_ok());
    }

    #[tokio::test]
    async fn test_get_count() {
        let engine = create_test_engine(2).await;

        assert!(engine.client.query("INSERT INTO test_table USING template TAGS (0, 0, 0 0) VALUES (NOW(), 0, 0)").await.is_ok());
        let count = engine.get_count("template").await;
        assert!(count.is_ok());
        assert_eq!(count.unwrap(), 1);

        assert!(engine.client.query("INSERT INTO test_table USING template TAGS (0, 0, 0, 0) VALUES (NOW() + 1s, 0, 0) (NOW() + 2s, 0, 0) (NOW() + 3s, 0, 0) (NOW() + 4s, 0, 0)").await.is_ok());
        let count = engine.get_count("template").await;
        assert!(count.is_ok());
        assert_eq!(count.unwrap(), 5);
    }
    #[tokio::test]
    async fn test_create_insert_and_count() {

        let mut fbb: FlatBufferBuilder = FlatBufferBuilder::new();
        let measurements_per_frame : usize = 16;
        let num_channels : usize = 4;
        
        let msg = generate_message(&mut fbb, measurements_per_frame,num_channels,&Malform::default());

        let mut engine = create_test_engine(4).await;

        assert_eq!(engine.get_count("template").await.unwrap(),0);

        assert!(engine.process_message(&msg).await.is_ok());
        assert_eq!(engine.post_message().await.unwrap(), measurements_per_frame);

        assert_eq!(engine.get_count("template").await.unwrap() as usize,measurements_per_frame);

        //  There should be no error reports filed
        assert::number_or_rows_with_errors_four_channel(&engine, 0).await;
        assert::correct_error_reports_filed(&engine, vec![]);
        
        engine.error.flush_reports(&msg.metadata(), msg.digitizer_id());
    }

    #[tokio::test]
    async fn test_create_insert() {

        let mut fbb: FlatBufferBuilder = FlatBufferBuilder::new();
        let measurements_per_frame : usize = 16;
        let num_channels : usize = 4;

        let msg = generate_message(&mut fbb, measurements_per_frame,num_channels,&Malform::default());

        let mut engine = create_test_engine(4).await;

        assert!(engine.process_message(&msg).await.is_ok());
        assert_eq!(engine.post_message().await.unwrap(), measurements_per_frame);

        assert::four_channels_correct(&engine,[
            assert::ChannelParameters{id: 0, samples : measurements_per_frame },
            assert::ChannelParameters{id: 1, samples : measurements_per_frame },
            assert::ChannelParameters{id: 2, samples : measurements_per_frame },
            assert::ChannelParameters{id: 3, samples : measurements_per_frame },
        ],measurements_per_frame,false).await;

        //  There should be no error reports filed
        assert::number_or_rows_with_errors_four_channel(&engine, 0).await;
        assert::correct_error_reports_filed(&engine, vec![]);
        engine.error.flush_reports(&msg.metadata(), msg.digitizer_id());
    }

    #[tokio::test]
    async fn test_create_insert_critical_malformed() {
        let mut fbb: FlatBufferBuilder = FlatBufferBuilder::new();
        let measurements_per_frame : usize = 16;
        let num_channels : usize = 2;
        
        //  Test that a message without a timestamp is not processed
        let msg = generate_message(&mut fbb, measurements_per_frame,num_channels,
            &vec![MalformType::DeleteTimestamp]);

        let mut engine = create_test_engine(2).await;

        assert!(engine.process_message(&msg).await.is_err());

        assert::number_or_rows_with_errors_two_channel(&engine, 0).await;
        assert::correct_error_reports_filed(&engine, vec![
            "Timestamp missing".to_owned(),
        ]);

        //  Test that a message without a channels is not processed
        let msg = generate_message(&mut fbb, measurements_per_frame,num_channels,
            &vec![MalformType::DeleteChannels]);

        assert!(engine.process_message(&msg).await.is_err());

        //  Test that a message without metadata is not processed
        //simulator::create_partly_random_message_with_now(&mut fbb, frame_number.clone(), digitizer_id.clone(), measurements_per_frame, num_channels, &vec![MalformType::DeleteMetadata]).unwrap();
        //let msg : DigitizerAnalogTraceMessage = root_as_digitizer_analog_trace_message(fbb.finished_data()).unwrap();
        //assert!(engine.process_message(&msg).await.is_err());

        assert::number_or_rows_with_errors_two_channel(&engine, 0).await;
        assert::correct_error_reports_filed(&engine, vec![
            "Channels missing".to_owned(),
            "Timestamp missing".to_owned(),
        ]);
        engine.error.flush_reports(&msg.metadata(), msg.digitizer_id());
    }

    #[tokio::test]
    async fn test_create_insert_malformed_missing_voltage() {

        let mut fbb: FlatBufferBuilder = FlatBufferBuilder::new();
        let measurements_per_frame : usize = 16;
        let num_channels : usize = 2;

        let msg = generate_message(&mut fbb, measurements_per_frame,num_channels,&vec![
            MalformType::DeleteVoltagesOfChannel(0)]);
        
        let mut engine = create_test_engine(2).await;

        assert!(engine.process_message(&msg).await.is_ok());
        assert_eq!(engine.post_message().await.unwrap(), measurements_per_frame);

        assert::two_channels_correct(&engine,[
            assert::ChannelParameters{id: 0, samples : 0 },
            assert::ChannelParameters{id: 1, samples : measurements_per_frame },
        ],measurements_per_frame,true).await;

        assert::number_or_rows_with_errors_two_channel(&engine, measurements_per_frame).await;
        assert::correct_error_reports_filed(&engine, vec![
            "Channel at index 0 has voltages null".to_owned(),
        ]);
        engine.error.flush_reports(&msg.metadata(), msg.digitizer_id());
    }

    #[tokio::test]
    async fn test_create_insert_malformed_truncated_voltages() {

        let mut fbb: FlatBufferBuilder = FlatBufferBuilder::new();
        let measurements_per_frame : usize = 16;
        let num_channels : usize = 2;
        
        //  Test that a message without missing samples is padded with zeros in the space
        let msg = generate_message(&mut fbb, measurements_per_frame,num_channels,&vec![
            MalformType::TruncateVoltagesOfChannelByHalf(0)]);
        
        let mut engine = create_test_engine(2).await;

        assert!(engine.process_message(&msg).await.is_ok());
        assert_eq!(engine.post_message().await.unwrap(), measurements_per_frame);

        assert::two_channels_correct(&engine,[
            assert::ChannelParameters{id: 0, samples : measurements_per_frame/2 },
            assert::ChannelParameters{id: 1, samples : measurements_per_frame },
        ],measurements_per_frame,true).await;

        assert::number_or_rows_with_errors_two_channel(&engine, measurements_per_frame).await;
        assert::correct_error_reports_filed(&engine, vec![
            format!("Channel at index 0 has incorrect sample count of {0}",measurements_per_frame/2),
        ]);
        engine.error.flush_reports(&msg.metadata(), msg.digitizer_id());
    }

    #[tokio::test]
    async fn test_create_insert_and_count_insufficient_channels() {

        let mut fbb: FlatBufferBuilder = FlatBufferBuilder::new();
        let measurements_per_frame : usize = 16;
        let num_channels : usize = 2;
        
        //  Test that a message without missing samples is passed with zeros in the space
        let msg = generate_message(&mut fbb, measurements_per_frame,num_channels,&Malform::default());

        let mut engine = create_test_engine(4).await;

        assert!(engine.process_message(&msg).await.is_ok());
        assert_eq!(engine.post_message().await.unwrap(), measurements_per_frame);

        assert::four_channels_correct(&engine,[
            assert::ChannelParameters{id: 0, samples : measurements_per_frame },
            assert::ChannelParameters{id: 1, samples : measurements_per_frame },
            assert::ChannelParameters{id: 0, samples : 0 },
            assert::ChannelParameters{id: 0, samples : 0 },
        ],measurements_per_frame,true).await;

        assert::number_or_rows_with_errors_four_channel(&engine, measurements_per_frame).await;
        assert::correct_error_reports_filed(&engine, vec![
            "Number of channels 2 insuffient, should be 4".to_owned(),
        ]);
        engine.error.flush_reports(&msg.metadata(), msg.digitizer_id());
    }

    #[tokio::test]
    async fn test_create_insert_and_count_excess_channels() {

        let mut fbb: FlatBufferBuilder = FlatBufferBuilder::new();
        let measurements_per_frame : usize = 16;
        let num_channels : usize = 8;
        
        //  Test that a message without missing samples is passed with zeros in the space
        let msg = generate_message(&mut fbb, measurements_per_frame,num_channels,&Malform::default());

        let mut engine = create_test_engine(4).await;

        assert!(engine.process_message(&msg).await.is_ok());
        assert_eq!(engine.post_message().await.unwrap(), measurements_per_frame);

        assert::four_channels_correct(&engine,[
            assert::ChannelParameters{id: 0, samples : measurements_per_frame },
            assert::ChannelParameters{id: 1, samples : measurements_per_frame },
            assert::ChannelParameters{id: 2, samples : measurements_per_frame },
            assert::ChannelParameters{id: 3, samples : measurements_per_frame },
        ],measurements_per_frame,true).await;

        assert::number_or_rows_with_errors_four_channel(&engine, measurements_per_frame).await;
        assert::correct_error_reports_filed(&engine, vec![
            "Number of channels 8 too large, only the first 4 channels retained".to_owned(),
        ]);
        engine.error.flush_reports(&msg.metadata(), msg.digitizer_id());
    }

    #[tokio::test]
    async fn test_create_insert_and_count_weird_but_unique_channel_id() {

        let mut fbb: FlatBufferBuilder = FlatBufferBuilder::new();
        let measurements_per_frame : usize = 16;
        let num_channels : usize = 4;
        
        //  Test that a message without missing samples is passed with zeros in the space
        let msg = generate_message(&mut fbb, measurements_per_frame,num_channels,
            &vec![
                MalformType::SetChannelId(0,156),
                MalformType::SetChannelId(1,36),
                MalformType::SetChannelId(2,136),
                MalformType::SetChannelId(3,6636)
            ]);

        let mut engine = create_test_engine(4).await;

        assert!(engine.process_message(&msg).await.is_ok());
        assert_eq!(engine.post_message().await.unwrap(), measurements_per_frame);

        assert::four_channels_correct(&engine,[
            assert::ChannelParameters{id: 156, samples : measurements_per_frame },
            assert::ChannelParameters{id: 36, samples : measurements_per_frame },
            assert::ChannelParameters{id: 136, samples : measurements_per_frame },
            assert::ChannelParameters{id: 6636, samples : measurements_per_frame },
        ],measurements_per_frame,false).await;

        //  There should be no error reports filed
        assert::number_or_rows_with_errors_four_channel(&engine, 0).await;
        assert::correct_error_reports_filed(&engine, vec![]);
        engine.error.flush_reports(&msg.metadata(), msg.digitizer_id());
    }

    #[tokio::test]
    async fn test_create_insert_and_count_duplicate_channel_id() {

        let mut fbb: FlatBufferBuilder = FlatBufferBuilder::new();
        let measurements_per_frame : usize = 16;
        let num_channels : usize = 4;
        
        //  Test that a message without missing samples is padded with zeros in the space
        let msg = generate_message(&mut fbb, measurements_per_frame,num_channels,
            &vec![
                MalformType::SetChannelId(0,56),
                MalformType::SetChannelId(1,9),
                MalformType::SetChannelId(2,66),
                MalformType::SetChannelId(3,66),
            ]);

        let mut engine = create_test_engine(4).await;

        assert!(engine.process_message(&msg).await.is_ok());
        assert_eq!(engine.post_message().await.unwrap(), measurements_per_frame);

        assert::four_channels_correct(&engine,[
            assert::ChannelParameters{id: 56, samples : measurements_per_frame },
            assert::ChannelParameters{id: 9, samples : measurements_per_frame },
            assert::ChannelParameters{id: 66, samples : measurements_per_frame },
            assert::ChannelParameters{id: 66, samples : measurements_per_frame },
        ],measurements_per_frame,true).await;

        assert::number_or_rows_with_errors_four_channel(&engine, measurements_per_frame).await;
        assert::correct_error_reports_filed(&engine, vec![
            "Channel at index 2 has duplicate channel identifier of 66".to_owned(),
            "Channel at index 3 has duplicate channel identifier of 66".to_owned(),
        ]);
        engine.error.flush_reports(&msg.metadata(), msg.digitizer_id());
    }

    #[tokio::test]
    async fn test_catastrophic() {
        let mut fbb: FlatBufferBuilder = FlatBufferBuilder::new();

        let measurements_per_frame : usize = 16;
        let num_channels : usize = 7;
        
        //  Test that a message without missing samples is padded with zeros in the space
        let msg = generate_message(&mut fbb, measurements_per_frame,num_channels,
            &vec![
                MalformType::SetChannelId(0,56),
                MalformType::SetChannelId(1,66),
                MalformType::SetChannelId(3,66),
                MalformType::TruncateVoltagesOfChannelByHalf(2),
                MalformType::DeleteVoltagesOfChannel(1),
            ]);
            
        let mut engine = create_test_engine(4).await;

        assert!(engine.process_message(&msg).await.is_ok());
        assert_eq!(engine.post_message().await.unwrap(), measurements_per_frame);

        assert::four_channels_correct(&engine,[
            assert::ChannelParameters{id: 56, samples : measurements_per_frame },
            assert::ChannelParameters{id: 66, samples : 0 },
            assert::ChannelParameters{id: 2, samples : measurements_per_frame/2 },
            assert::ChannelParameters{id: 66, samples : measurements_per_frame },
        ],measurements_per_frame,true).await;

        assert::number_or_rows_with_errors_four_channel(&engine, measurements_per_frame).await;
        assert::correct_error_reports_filed(&engine, vec![
            "Channel at index 1 has duplicate channel identifier of 66".to_owned(),
            "Channel at index 3 has duplicate channel identifier of 66".to_owned(),
            "Number of channels 7 too large, only the first 4 channels retained".to_owned(),
            "Channel at index 1 has voltages null".to_owned(),
            format!("Channel at index 2 has incorrect sample count of {0}",measurements_per_frame/2),
        ]);
        engine.error.flush_reports(&msg.metadata(), msg.digitizer_id());
    }
}
