use super::{
    error_reporter::TDEngineErrorReporter,
    framedata::FrameData,
    views::{create_column_views, create_frame_column_views},
    StatementErrorCode, TDEngineError, TimeSeriesEngine, TraceMessageErrorCode,
};
use anyhow::{Error, Result};
use async_trait::async_trait;
use supermusr_streaming_types::dat1_digitizer_analog_trace_v1_generated::DigitizerAnalogTraceMessage;
use taos::{AsyncBindable, AsyncQueryable, AsyncTBuilder, Stmt, Taos, TaosBuilder, Value};
use tracing::debug;

pub(crate) struct TDEngine {
    client: Taos,
    database: String,
    stmt: Stmt,
    frame_stmt: Stmt,
    error: TDEngineErrorReporter,
    frame_data: FrameData,
}

impl TDEngine {
    pub(crate) async fn from_optional(
        broker: String,
        username: Option<String>,
        password: Option<String>,
        database: String,
    ) -> Result<Self, Error> {
        let url = match Option::zip(username, password) {
            Some((username, password)) => format!("taos+ws://{broker}@{username}:{password}"),
            None => format!("taos+ws://{broker}"),
        };

        debug!("Creating TaosBuilder with url {url}");
        let client = TaosBuilder::from_dsn(url)
            .map_err(TDEngineError::TaosBuilder)?
            .build()
            .await
            .map_err(TDEngineError::TaosBuilder)?;

        let stmt = Stmt::init(&client)
            .await
            .map_err(|e| TDEngineError::TaosStmt(StatementErrorCode::Init, e))?;

        let frame_stmt = Stmt::init(&client)
            .await
            .map_err(TDEngineError::TaosBuilder)?;

        Ok(TDEngine {
            client,
            database,
            stmt,
            frame_stmt,
            error: TDEngineErrorReporter::new(),
            frame_data: FrameData::default(),
        })
    }

    pub(crate) async fn create_database(&self) -> Result<(), TDEngineError> {
        self.client
            .exec(&format!(
                "CREATE DATABASE IF NOT EXISTS {} PRECISION 'ns'",
                self.database
            ))
            .await
            .map_err(TDEngineError::TaosBuilder)?;

        self.client
            .use_database(&self.database)
            .await
            .map_err(TDEngineError::TaosBuilder)
    }

    pub(crate) async fn init_with_channel_count(
        &mut self,
        num_channels: usize,
    ) -> Result<(), TDEngineError> {
        self.frame_data.set_channel_count(num_channels);
        self.create_supertable().await?;

        let template_table = self.database.to_owned() + ".template";
        let stmt_sql = format!(
            "INSERT INTO ? USING {template_table} TAGS (?) VALUES (?{0})",
            ", ?".repeat(num_channels)
        );

        self.stmt
            .prepare(&stmt_sql)
            .await
            .map_err(|e| TDEngineError::TaosStmt(StatementErrorCode::Prepare, e))?;

        let frame_template_table = self.database.to_owned() + ".frame_template";
        let frame_stmt_sql = format!(
            "INSERT INTO ? USING {frame_template_table} TAGS (?) VALUES (?, ?, ?, ?, ?{0})",
            ", ?".repeat(num_channels)
        );

        self.frame_stmt
            .prepare(&frame_stmt_sql)
            .await
            .map_err(|e| TDEngineError::TaosStmt(StatementErrorCode::Prepare, e))?;
        Ok(())
    }

    async fn create_supertable(&self) -> Result<(), TDEngineError> {
        let metrics_string = format!(
            "ts TIMESTAMP{0}",
            (0..self.frame_data.num_channels)
                .map(|ch| format!(", c{ch} SMALLINT UNSIGNED"))
                .fold(String::new(), |a, b| a + &b)
        );
        let template_table = self.database.to_owned() + ".template";
        let string = format!("CREATE STABLE IF NOT EXISTS {template_table} ({metrics_string}) TAGS (digitizer_id TINYINT UNSIGNED)");
        self.client
            .exec(&string)
            .await
            .map_err(|e| TDEngineError::SqlError(string.clone(), e))?;

        let frame_metrics_string = format!("frame_ts TIMESTAMP, sample_count INT UNSIGNED, sampling_rate INT UNSIGNED, frame_number INT UNSIGNED, error_code INT UNSIGNED{0}",
            (0..self.frame_data.num_channels)
                .map(|ch|format!(", cid{ch} INT UNSIGNED"))
                .fold(String::new(),|a,b|a + &b)
        );
        let frame_template_table = self.database.to_owned() + ".frame_template";
        let string = format!("CREATE STABLE IF NOT EXISTS {frame_template_table} ({frame_metrics_string}) TAGS (digitizer_id TINYINT UNSIGNED)");
        self.client
            .exec(&string)
            .await
            .map_err(|e| TDEngineError::SqlError(string.clone(), e))?;
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
        // Obtain the channel data, and error check
        self.error.test_metadata(message);

        // Obtain message data, and error check
        self.frame_data.init(message)?;

        // Obtain the channel data, and error check
        self.error
            .test_channels(&self.frame_data, &message.channels().unwrap());

        let mut table_name = self.frame_data.get_table_name();
        let mut frame_table_name = self.frame_data.get_frame_table_name();
        frame_table_name.insert(0, '.');
        frame_table_name.insert_str(0, &self.database);
        table_name.insert(0, '.');
        table_name.insert_str(0, &self.database);
        let channels = message.channels().ok_or(TDEngineError::TraceMessage(
            TraceMessageErrorCode::ChannelsMissing,
        ))?;
        let frame_column_views =
            create_frame_column_views(&self.frame_data, &self.error, &channels).unwrap();
        let column_views = create_column_views(&self.frame_data, &channels).unwrap();
        let tags = [Value::UTinyInt(self.frame_data.digitizer_id)];

        //  Initialise Statement
        self.stmt
            .set_tbname(&table_name)
            .await
            .map_err(|e| TDEngineError::TaosStmt(StatementErrorCode::SetTableName, e))
            .unwrap();
        self.stmt
            .set_tags(&tags)
            .await
            .map_err(|e| TDEngineError::TaosStmt(StatementErrorCode::SetTags, e))
            .unwrap();
        self.stmt
            .bind(&column_views)
            .await
            .map_err(|e| TDEngineError::TaosStmt(StatementErrorCode::Bind, e))
            .unwrap();
        self.stmt
            .add_batch()
            .await
            .map_err(|e| TDEngineError::TaosStmt(StatementErrorCode::AddBatch, e))
            .unwrap();

        self.frame_stmt
            .set_tbname(&frame_table_name)
            .await
            .map_err(|e| TDEngineError::TaosStmt(StatementErrorCode::SetTableName, e))?;
        self.frame_stmt
            .set_tags(&tags)
            .await
            .map_err(|e| TDEngineError::TaosStmt(StatementErrorCode::SetTags, e))?;
        self.frame_stmt
            .bind(&frame_column_views)
            .await
            .map_err(|e| TDEngineError::TaosStmt(StatementErrorCode::Bind, e))?;
        self.frame_stmt
            .add_batch()
            .await
            .map_err(|e| TDEngineError::TaosStmt(StatementErrorCode::AddBatch, e))?;
        Ok(())
    }

    /// Sends data extracted from a previous call to ``process_message`` to the tdengine server.
    /// #Returns
    /// The number of rows affected by the post or an error
    async fn post_message(&mut self) -> Result<usize> {
        let result = self
            .stmt
            .execute()
            .await
            .map_err(|e| TDEngineError::TaosStmt(StatementErrorCode::Execute, e))?
            + self
                .frame_stmt
                .execute()
                .await
                .map_err(|e| TDEngineError::TaosStmt(StatementErrorCode::Execute, e))?;
        Ok(result)
    }
}
