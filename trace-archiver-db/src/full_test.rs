// Note these tests should be run in single-threaded mode, in an environment with a .env file in the working directory and a TDEngine server on the network.
// cargo test -- --test-threads 1 --show-output --include-ignored
#[cfg(test)]
mod test {
    use anyhow::Result;
    use chrono::{DateTime, Utc};
    use common::{Channel, FrameNumber, Intensity};

    use flatbuffers::FlatBufferBuilder;
    use serde::de::DeserializeOwned;
    use taos::*;

    use streaming_types::dat1_digitizer_analog_trace_v1_generated::{
        root_as_digitizer_analog_trace_message, DigitizerAnalogTraceMessage,
    };

    use tdengine::{error_reporter::ErrorCode, tdengine::TDEngine, TimeSeriesEngine};
    use trace_simulator::{self, Malform, MalformType};

    #[derive(Debug, serde::Deserialize)]
    struct SingleI32QueryRecord(i32);
    #[derive(Debug, serde::Deserialize)]
    struct TwoChannelTraceQueryRecord {
        _ts: DateTime<Utc>,
        _frametime: DateTime<Utc>,
        c0: Intensity,
        c1: Intensity,
    }
    #[derive(Debug, serde::Deserialize)]
    struct TwoChannelFrameQueryRecord {
        _frame_ts: DateTime<Utc>,
        _sample_count: u32,
        _sampling_rate: u32,
        _frame_number: FrameNumber,
        error_code: u32,
        cid0: Channel,
        cid1: Channel,
    }

    #[derive(Debug, serde::Deserialize)]
    struct FourChannelTraceQueryRecord {
        _ts: DateTime<Utc>,
        _frametime: DateTime<Utc>,
        c0: Intensity,
        c1: Intensity,
        c2: Intensity,
        c3: Intensity,
    }
    #[derive(Debug, serde::Deserialize)]
    struct FourChannelFrameQueryRecord {
        _frame_ts: DateTime<Utc>,
        _sample_count: u32,
        _sampling_rate: u32,
        _frame_number: FrameNumber,
        error_code: u32,
        cid0: Channel,
        cid1: Channel,
        cid2: Channel,
        cid3: Channel,
    }

    async fn delete_test_database(engine: &mut TDEngine) -> Result<()> {
        engine
            .exec("DROP DATABASE IF EXISTS test_database")
            .await
            .unwrap();
        Ok(())
    }
    async fn create_test_database(engine: &mut TDEngine) -> Result<()> {
        engine
            .exec("CREATE DATABASE IF NOT EXISTS test_database PRECISION 'ns'")
            .await
            .unwrap();
        engine.use_database("test_database").await.unwrap();
        Ok(())
    }

    ///
    async fn channel_query<D>(engine: &mut TDEngine, sql: &str) -> Vec<D>
    where
        D: DeserializeOwned,
    {
        engine
            .query(sql)
            .await
            .unwrap()
            .deserialize()
            .try_collect()
            .await
            .unwrap()
    }

    ///
    async fn get_count(engine: &mut TDEngine, table: &str) -> Result<i32> {
        let sql = format!("SELECT COUNT(*) FROM {table}");
        let count_vec = channel_query::<SingleI32QueryRecord>(engine, &sql).await;
        if count_vec.is_empty() {
            panic!("{sql} returned empty count")
        } else {
            Ok(count_vec[0].0)
        }
    }

    ///  A module of functions that test assertions
    mod assert {
        use super::*;

        pub(super) struct ChannelParameters {
            pub(super) id: Channel,
            pub(super) samples: usize,
        }
        pub(super) async fn two_channels_correct(
            engine: &mut TDEngine,
            channel: [ChannelParameters; 2],
            measurements_per_frame: usize,
            error_code: u32,
        ) {
            let results =
                channel_query::<TwoChannelFrameQueryRecord>(engine, "SELECT * FROM frame_template")
                    .await;
            assert_eq!(results.len(), 1);
            for result in &results {
                for n in 0..2 {
                    let ids = [result.cid0, result.cid1];
                    assert_eq!(ids[n], channel[n].id, "Failed with record {result:?}");
                }
                assert_eq!(
                    result.error_code, error_code,
                    "Failed with record {result:?}"
                );
            }

            let results =
                channel_query::<TwoChannelTraceQueryRecord>(engine, "SELECT * FROM template").await;
            assert_eq!(results.len(), measurements_per_frame);
            for (i, result) in results.iter().enumerate() {
                for n in 0..2 {
                    if i >= channel[n].samples {
                        let intensities = [result.c0, result.c1];
                        assert_eq!(intensities[n], 0);
                    }
                }
            }
        }

        pub(super) async fn four_channels_correct(
            engine: &mut TDEngine,
            channel: [ChannelParameters; 4],
            measurements_per_frame: usize,
            error_code: u32,
        ) {
            let results = channel_query::<FourChannelFrameQueryRecord>(
                engine,
                "SELECT * FROM frame_template",
            )
            .await;
            assert_eq!(results.len(), 1);
            for result in &results {
                for n in 0..4 {
                    let ids = [result.cid0, result.cid1, result.cid2, result.cid3];
                    assert_eq!(ids[n], channel[n].id, "Failed with record {result:?}");
                }
                assert_eq!(
                    result.error_code, error_code,
                    "Failed with record {result:?}"
                );
            }

            let results =
                channel_query::<FourChannelTraceQueryRecord>(engine, "SELECT * FROM template")
                    .await;
            assert_eq!(results.len(), measurements_per_frame);
            for (i, result) in results.iter().enumerate() {
                for n in 0..4 {
                    if i >= channel[n].samples {
                        let intensities = [result.c0, result.c1, result.c2, result.c3];
                        assert_eq!(intensities[n], 0);
                    }
                }
            }
        }

        pub(super) fn correct_error_reports_filed(
            engine: &mut TDEngine,
            expected_reports: Vec<String>,
        ) {
            assert_eq!(
                expected_reports.len(),
                engine.get_error_reporter().num_errors()
            );
            for error_result in engine.get_error_reporter().reports_iter() {
                assert!(expected_reports.contains(error_result));
            }
        }

        pub(super) async fn number_or_rows_with_errors<D>(
            engine: &mut TDEngine,
            expected_rows_with_error: usize,
        ) where
            D: DeserializeOwned,
        {
            let results =
                channel_query::<D>(engine, "SELECT * FROM frame_template WHERE error_code <> 0")
                    .await;
            assert_eq!(results.len(), expected_rows_with_error);
        }
    }

    // Functions used in every test for setup

    ///Creates a new tdengine instance and initialises it in test mode with the given number of channels
    /// #Arguments
    /// *num_channels - Number of channels
    /// #Returns
    /// A TDEngine instance
    async fn create_test_engine(num_channels: usize) -> TDEngine {
        let mut engine = TDEngine::from_optional(
            dotenv::var("TDENGINE_URL").ok(),
            dotenv::var("TDENGINE_PORT")
                .ok()
                .and_then(|e| str::parse::<u32>(&e).ok()),
            dotenv::var("TDENGINE_USER").ok(),
            dotenv::var("TDENGINE_PASSWORD").ok(),
            dotenv::var("TDENGINE_DATABASE").ok(),
        )
        .await
        .unwrap();
        delete_test_database(&mut engine).await.unwrap();
        create_test_database(&mut engine).await.unwrap();
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
    fn generate_message<'b, 'a: 'b>(
        fbb: &'a mut FlatBufferBuilder,
        measurements_per_frame: usize,
        num_channels: usize,
        malform: &Malform,
    ) -> DigitizerAnalogTraceMessage<'b> {
        let digitizer_id = 0..=24;
        let frame_number = 0..=5;

        trace_simulator::create_partly_random_message_with_now(
            fbb,
            frame_number,
            digitizer_id,
            measurements_per_frame,
            num_channels,
            malform,
        )
        .unwrap();
        root_as_digitizer_analog_trace_message(fbb.finished_data()).unwrap()
    }

    #[tokio::test]
    #[ignore]
    async fn test_create_db() {
        let mut engine = TDEngine::from_optional(
            dotenv::var("TDENGINE_URL").ok(),
            dotenv::var("TDENGINE_PORT")
                .ok()
                .and_then(|e| str::parse::<u32>(&e).ok()),
            dotenv::var("TDENGINE_USER").ok(),
            dotenv::var("TDENGINE_PASSWORD").ok(),
            dotenv::var("TDENGINE_DATABASE").ok(),
        )
        .await
        .unwrap();
        assert!(delete_test_database(&mut engine).await.is_ok());
        assert!(create_test_database(&mut engine).await.is_ok());
    }

    #[tokio::test]
    #[ignore]
    async fn test_create_tables() {
        let mut engine = TDEngine::from_optional(
            dotenv::var("TDENGINE_URL").ok(),
            dotenv::var("TDENGINE_PORT")
                .ok()
                .and_then(|e| str::parse::<u32>(&e).ok()),
            dotenv::var("TDENGINE_USER").ok(),
            dotenv::var("TDENGINE_PASSWORD").ok(),
            dotenv::var("TDENGINE_DATABASE").ok(),
        )
        .await
        .unwrap();
        assert!(delete_test_database(&mut engine).await.is_ok());
        assert!(create_test_database(&mut engine).await.is_ok());
        assert!(engine.init_with_channel_count(8).await.is_ok());
    }

    #[tokio::test]
    #[ignore]
    async fn test_get_count() {
        let mut engine = create_test_engine(2).await;

        assert!(engine
            .query("INSERT INTO test_table USING template TAGS (0) VALUES (NOW(), 0, 0, 0)")
            .await
            .is_ok());
        let count = get_count(&mut engine, "template").await;
        assert!(count.is_ok());
        assert_eq!(count.unwrap(), 1);

        assert!(engine
            .query("INSERT INTO test_table USING template TAGS (0) VALUES (NOW() + 1s, 0, 0, 0) (NOW() + 2s, 0, 0, 0) (NOW() + 3s, 0, 0, 0) (NOW() + 4s, 0, 0, 0)")
            .await
            .is_ok());
        let count = get_count(&mut engine, "template").await;
        assert!(count.is_ok());
        assert_eq!(count.unwrap(), 5);
    }

    #[tokio::test]
    #[ignore]
    async fn test_create_insert_and_count() {
        let mut fbb: FlatBufferBuilder = FlatBufferBuilder::new();
        let measurements_per_frame: usize = 16;
        let num_channels: usize = 4;

        let msg = generate_message(
            &mut fbb,
            measurements_per_frame,
            num_channels,
            &Malform::default(),
        );

        let mut engine = create_test_engine(4).await;

        assert_eq!(get_count(&mut engine, "template").await.unwrap(), 0);

        assert!(engine.process_message(&msg).await.is_ok());
        assert_eq!(
            engine.post_message().await.unwrap(),
            measurements_per_frame + 1
        );

        assert_eq!(
            get_count(&mut engine, "template").await.unwrap() as usize,
            measurements_per_frame
        );

        //  There should be no error reports filed
        assert::number_or_rows_with_errors::<FourChannelFrameQueryRecord>(&mut engine, 0).await;
        assert::correct_error_reports_filed(&mut engine, vec![]);

        engine
            .get_error_reporter()
            .flush_reports(&msg.metadata(), msg.digitizer_id());
    }

    #[tokio::test]
    #[ignore]
    async fn test_create_insert() {
        let mut fbb: FlatBufferBuilder = FlatBufferBuilder::new();
        let measurements_per_frame: usize = 16;
        let num_channels: usize = 4;

        let msg = generate_message(
            &mut fbb,
            measurements_per_frame,
            num_channels,
            &Malform::default(),
        );

        let mut engine = create_test_engine(4).await;

        assert!(engine.process_message(&msg).await.is_ok());
        assert_eq!(
            engine.post_message().await.unwrap(),
            measurements_per_frame + 1
        );

        assert::four_channels_correct(
            &mut engine,
            [
                assert::ChannelParameters {
                    id: 0,
                    samples: measurements_per_frame,
                },
                assert::ChannelParameters {
                    id: 1,
                    samples: measurements_per_frame,
                },
                assert::ChannelParameters {
                    id: 2,
                    samples: measurements_per_frame,
                },
                assert::ChannelParameters {
                    id: 3,
                    samples: measurements_per_frame,
                },
            ],
            measurements_per_frame,
            tdengine::error_reporter::ErrorCode::NoError as u32,
        )
        .await;

        //  There should be no error reports filed
        assert::number_or_rows_with_errors::<FourChannelFrameQueryRecord>(&mut engine, 0).await;
        assert::correct_error_reports_filed(&mut engine, vec![]);
        engine
            .get_error_reporter()
            .flush_reports(&msg.metadata(), msg.digitizer_id());
    }

    #[tokio::test]
    #[ignore]
    async fn test_create_insert_critical_malformed() {
        let mut fbb: FlatBufferBuilder = FlatBufferBuilder::new();
        let measurements_per_frame: usize = 16;
        let num_channels: usize = 2;

        //  Test that a message without a timestamp is not processed
        let msg = generate_message(
            &mut fbb,
            measurements_per_frame,
            num_channels,
            &vec![MalformType::DeleteTimestamp],
        );

        let mut engine = create_test_engine(2).await;

        assert!(engine.process_message(&msg).await.is_err());

        assert::number_or_rows_with_errors::<TwoChannelFrameQueryRecord>(&mut engine, 0).await;
        assert::correct_error_reports_filed(&mut engine, vec!["Timestamp missing".to_owned()]);

        //  Test that a message without a channels is not processed
        let msg = generate_message(
            &mut fbb,
            measurements_per_frame,
            num_channels,
            &vec![MalformType::DeleteChannels],
        );

        assert!(engine.process_message(&msg).await.is_err());

        //  Test that a message without metadata is not processed
        //simulator::create_partly_random_message_with_now(&mut fbb, frame_number.clone(), digitizer_id.clone(), measurements_per_frame, num_channels, &vec![MalformType::DeleteMetadata]).unwrap();
        //let msg : DigitizerAnalogTraceMessage = root_as_digitizer_analog_trace_message(fbb.finished_data()).unwrap();
        //assert!(engine.process_message(&msg).await.is_err());

        assert::number_or_rows_with_errors::<TwoChannelFrameQueryRecord>(&mut engine, 0).await;
        assert::correct_error_reports_filed(
            &mut engine,
            vec![
                "Channels missing".to_owned(),
                "Timestamp missing".to_owned(),
            ],
        );
        engine
            .get_error_reporter()
            .flush_reports(&msg.metadata(), msg.digitizer_id());
    }

    #[tokio::test]
    #[ignore]
    async fn test_create_insert_malformed_missing_voltage() {
        let mut fbb: FlatBufferBuilder = FlatBufferBuilder::new();
        let measurements_per_frame: usize = 16;
        let num_channels: usize = 2;

        let msg = generate_message(
            &mut fbb,
            measurements_per_frame,
            num_channels,
            &vec![MalformType::DeleteVoltagesOfChannel(0)],
        );

        let mut engine = create_test_engine(2).await;

        assert!(engine.process_message(&msg).await.is_ok());
        assert_eq!(
            engine.post_message().await.unwrap(),
            measurements_per_frame + 1
        );

        assert::two_channels_correct(
            &mut engine,
            [
                assert::ChannelParameters { id: 0, samples: 0 },
                assert::ChannelParameters {
                    id: 1,
                    samples: measurements_per_frame,
                },
            ],
            measurements_per_frame,
            tdengine::error_reporter::ErrorCode::ChannelVoltagesMissing as u32,
        )
        .await;

        assert::number_or_rows_with_errors::<TwoChannelFrameQueryRecord>(&mut engine, 1).await;
        assert::correct_error_reports_filed(
            &mut engine,
            vec!["Channel at index 0 has voltages null".to_owned()],
        );
        engine
            .get_error_reporter()
            .flush_reports(&msg.metadata(), msg.digitizer_id());
    }

    #[tokio::test]
    #[ignore]
    async fn test_create_insert_malformed_truncated_voltages() {
        let mut fbb: FlatBufferBuilder = FlatBufferBuilder::new();
        let measurements_per_frame: usize = 16;
        let num_channels: usize = 2;

        //  Test that a message without missing samples is padded with zeros in the space
        let msg = generate_message(
            &mut fbb,
            measurements_per_frame,
            num_channels,
            &vec![MalformType::TruncateVoltagesOfChannelByHalf(0)],
        );

        let mut engine = create_test_engine(2).await;

        assert!(engine.process_message(&msg).await.is_ok());
        assert_eq!(
            engine.post_message().await.unwrap(),
            measurements_per_frame + 1 // one extra for the frame table
        );

        assert::two_channels_correct(
            &mut engine,
            [
                assert::ChannelParameters {
                    id: 0,
                    samples: measurements_per_frame / 2,
                },
                assert::ChannelParameters {
                    id: 1,
                    samples: measurements_per_frame,
                },
            ],
            measurements_per_frame,
            tdengine::error_reporter::ErrorCode::NumSamplesIncorrect as u32,
        )
        .await;

        assert::number_or_rows_with_errors::<TwoChannelFrameQueryRecord>(&mut engine, 1).await;
        assert::correct_error_reports_filed(
            &mut engine,
            vec![format!(
                "Channel at index 0 has incorrect sample count of {0}",
                measurements_per_frame / 2
            )],
        );
        engine
            .get_error_reporter()
            .flush_reports(&msg.metadata(), msg.digitizer_id());
    }

    #[tokio::test]
    #[ignore]
    async fn test_create_insert_and_count_insufficient_channels() {
        let mut fbb: FlatBufferBuilder = FlatBufferBuilder::new();
        let measurements_per_frame: usize = 16;
        let num_channels: usize = 2;

        //  Test that a message without missing samples is passed with zeros in the space
        let msg = generate_message(
            &mut fbb,
            measurements_per_frame,
            num_channels,
            &Malform::default(),
        );

        let mut engine = create_test_engine(4).await;

        assert!(engine.process_message(&msg).await.is_ok());
        assert_eq!(
            engine.post_message().await.unwrap(),
            measurements_per_frame + 1
        );

        assert::four_channels_correct(
            &mut engine,
            [
                assert::ChannelParameters {
                    id: 0,
                    samples: measurements_per_frame,
                },
                assert::ChannelParameters {
                    id: 1,
                    samples: measurements_per_frame,
                },
                assert::ChannelParameters { id: 0, samples: 0 },
                assert::ChannelParameters { id: 0, samples: 0 },
            ],
            measurements_per_frame,
            ErrorCode::NumChannelsIncorrect as u32,
        )
        .await;

        assert::number_or_rows_with_errors::<FourChannelFrameQueryRecord>(&mut engine, 1).await;
        assert::correct_error_reports_filed(
            &mut engine,
            vec!["Number of channels 2 insuffient, should be 4".to_owned()],
        );
        engine
            .get_error_reporter()
            .flush_reports(&msg.metadata(), msg.digitizer_id());
    }

    #[tokio::test]
    #[ignore]
    async fn test_create_insert_and_count_excess_channels() {
        let mut fbb: FlatBufferBuilder = FlatBufferBuilder::new();
        let measurements_per_frame: usize = 16;
        let num_channels: usize = 8;

        //  Test that a message without missing samples is passed with zeros in the space
        let msg = generate_message(
            &mut fbb,
            measurements_per_frame,
            num_channels,
            &Malform::default(),
        );

        let mut engine = create_test_engine(4).await;

        assert!(engine.process_message(&msg).await.is_ok());
        assert_eq!(
            engine.post_message().await.unwrap(),
            measurements_per_frame + 1
        );

        assert::four_channels_correct(
            &mut engine,
            [
                assert::ChannelParameters {
                    id: 0,
                    samples: measurements_per_frame,
                },
                assert::ChannelParameters {
                    id: 1,
                    samples: measurements_per_frame,
                },
                assert::ChannelParameters {
                    id: 2,
                    samples: measurements_per_frame,
                },
                assert::ChannelParameters {
                    id: 3,
                    samples: measurements_per_frame,
                },
            ],
            measurements_per_frame,
            ErrorCode::NumChannelsIncorrect as u32,
        )
        .await;

        assert::number_or_rows_with_errors::<FourChannelFrameQueryRecord>(&mut engine, 1).await;
        assert::correct_error_reports_filed(
            &mut engine,
            vec!["Number of channels 8 too large, only the first 4 channels retained".to_owned()],
        );
        engine
            .get_error_reporter()
            .flush_reports(&msg.metadata(), msg.digitizer_id());
    }

    #[tokio::test]
    #[ignore]
    async fn test_create_insert_and_count_weird_but_unique_channel_id() {
        let mut fbb: FlatBufferBuilder = FlatBufferBuilder::new();
        let measurements_per_frame: usize = 16;
        let num_channels: usize = 4;

        //  Test that a message without missing samples is passed with zeros in the space
        let msg = generate_message(
            &mut fbb,
            measurements_per_frame,
            num_channels,
            &vec![
                MalformType::SetChannelId(0, 156),
                MalformType::SetChannelId(1, 36),
                MalformType::SetChannelId(2, 136),
                MalformType::SetChannelId(3, 6636),
            ],
        );

        let mut engine = create_test_engine(4).await;

        assert!(engine.process_message(&msg).await.is_ok());
        assert_eq!(
            engine.post_message().await.unwrap(),
            measurements_per_frame + 1
        );

        assert::four_channels_correct(
            &mut engine,
            [
                assert::ChannelParameters {
                    id: 156,
                    samples: measurements_per_frame,
                },
                assert::ChannelParameters {
                    id: 36,
                    samples: measurements_per_frame,
                },
                assert::ChannelParameters {
                    id: 136,
                    samples: measurements_per_frame,
                },
                assert::ChannelParameters {
                    id: 6636,
                    samples: measurements_per_frame,
                },
            ],
            measurements_per_frame,
            ErrorCode::NoError as u32,
        )
        .await;

        //  There should be no error reports filed
        assert::number_or_rows_with_errors::<FourChannelFrameQueryRecord>(&mut engine, 0).await;
        assert::correct_error_reports_filed(&mut engine, vec![]);
        engine
            .get_error_reporter()
            .flush_reports(&msg.metadata(), msg.digitizer_id());
    }

    #[tokio::test]
    #[ignore]
    async fn test_create_insert_and_count_duplicate_channel_id() {
        let mut fbb: FlatBufferBuilder = FlatBufferBuilder::new();
        let measurements_per_frame: usize = 16;
        let num_channels: usize = 4;

        //  Test that a message without missing samples is padded with zeros in the space
        let msg = generate_message(
            &mut fbb,
            measurements_per_frame,
            num_channels,
            &vec![
                MalformType::SetChannelId(0, 56),
                MalformType::SetChannelId(1, 9),
                MalformType::SetChannelId(2, 66),
                MalformType::SetChannelId(3, 66),
            ],
        );

        let mut engine = create_test_engine(4).await;

        assert!(engine.process_message(&msg).await.is_ok());
        assert_eq!(
            engine.post_message().await.unwrap(),
            measurements_per_frame + 1
        );

        assert::four_channels_correct(
            &mut engine,
            [
                assert::ChannelParameters {
                    id: 56,
                    samples: measurements_per_frame,
                },
                assert::ChannelParameters {
                    id: 9,
                    samples: measurements_per_frame,
                },
                assert::ChannelParameters {
                    id: 66,
                    samples: measurements_per_frame,
                },
                assert::ChannelParameters {
                    id: 66,
                    samples: measurements_per_frame,
                },
            ],
            measurements_per_frame,
            ErrorCode::DuplicateChannelIds as u32,
        )
        .await;

        assert::number_or_rows_with_errors::<FourChannelFrameQueryRecord>(&mut engine, 1).await;
        assert::correct_error_reports_filed(
            &mut engine,
            vec![
                "Channel at index 2 has duplicate channel identifier of 66".to_owned(),
                "Channel at index 3 has duplicate channel identifier of 66".to_owned(),
            ],
        );
        engine
            .get_error_reporter()
            .flush_reports(&msg.metadata(), msg.digitizer_id());
    }

    #[tokio::test]
    #[ignore]
    async fn test_catastrophic() {
        let mut fbb: FlatBufferBuilder = FlatBufferBuilder::new();

        let measurements_per_frame: usize = 16;
        let num_channels: usize = 7;

        //  Test that a message without missing samples is padded with zeros in the space
        let msg = generate_message(
            &mut fbb,
            measurements_per_frame,
            num_channels,
            &vec![
                MalformType::SetChannelId(0, 56),
                MalformType::SetChannelId(1, 66),
                MalformType::SetChannelId(3, 66),
                MalformType::TruncateVoltagesOfChannelByHalf(2),
                MalformType::DeleteVoltagesOfChannel(1),
            ],
        );

        let mut engine = create_test_engine(4).await;

        assert!(engine.process_message(&msg).await.is_ok());
        assert_eq!(
            engine.post_message().await.unwrap(),
            measurements_per_frame + 1
        );

        assert::four_channels_correct(
            &mut engine,
            [
                assert::ChannelParameters {
                    id: 56,
                    samples: measurements_per_frame,
                },
                assert::ChannelParameters { id: 66, samples: 0 },
                assert::ChannelParameters {
                    id: 2,
                    samples: measurements_per_frame / 2,
                },
                assert::ChannelParameters {
                    id: 66,
                    samples: measurements_per_frame,
                },
            ],
            measurements_per_frame,
            ErrorCode::DuplicateChannelIds as u32
                | ErrorCode::NumChannelsIncorrect as u32
                | ErrorCode::ChannelVoltagesMissing as u32
                | ErrorCode::NumSamplesIncorrect as u32,
        )
        .await;

        assert::number_or_rows_with_errors::<FourChannelFrameQueryRecord>(&mut engine, 1).await;
        assert::correct_error_reports_filed(
            &mut engine,
            vec![
                "Channel at index 1 has duplicate channel identifier of 66".to_owned(),
                "Channel at index 3 has duplicate channel identifier of 66".to_owned(),
                "Number of channels 7 too large, only the first 4 channels retained".to_owned(),
                "Channel at index 1 has voltages null".to_owned(),
                format!(
                    "Channel at index 2 has incorrect sample count of {0}",
                    measurements_per_frame / 2
                ),
            ],
        );
        engine
            .get_error_reporter()
            .flush_reports(&msg.metadata(), msg.digitizer_id());
    }
}
