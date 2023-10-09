use anyhow::Result;
use async_trait::async_trait;

use taos::*;

use streaming_types::dat1_digitizer_analog_trace_v1_generated::DigitizerAnalogTraceMessage;

use crate::error::{SQLError, StatementError, TDEngineError};

use super::error::{self, Error};
use super::tdengine_views as views;

use super::{
    error_reporter::TDEngineErrorReporter, framedata::FrameData, tdengine_login::TDEngineLogin,
    TimeSeriesEngine,
};

pub struct TDEngine {
    login: TDEngineLogin,
    client: Taos,
    stmt: Stmt,
    frame_stmt: Stmt,
    error: TDEngineErrorReporter,
    frame_data: FrameData,
}

impl TDEngine {
    pub async fn from_optional(
        url: Option<String>,
        port: Option<u32>,
        username: Option<String>,
        password: Option<String>,
        database: Option<String>,
    ) -> Result<Self, Error> {
        let login = TDEngineLogin::from_optional(url, port, username, password, database)?;
        log::debug!("Creating TaosBuilder with login {login:?}");
        let client = TaosBuilder::from_dsn(login.get_url())
            .map_err(TDEngineError::TaosBuilder)?
            .build()
            .await
            .map_err(TDEngineError::TaosBuilder)?;
        let stmt = Stmt::init(&client).await.map_err(TDEngineError::TaosBuilder)?;
        let frame_stmt = Stmt::init(&client).await.map_err(TDEngineError::TaosBuilder)?;
        Ok(TDEngine {
            login,
            client,
            stmt,
            frame_stmt,
            error: TDEngineErrorReporter::new(),
            frame_data: FrameData::default(),
        })
    }

    pub async fn delete_database(&self) -> Result<()> {
        self.client
            .exec(&format!(
                "DROP DATABASE IF EXISTS {}",
                self.login.get_database()
            ))
            .await?;
        Ok(())
    }

    pub async fn create_database(&self) -> Result<()> {
        self.client
            .exec(&format!(
                "CREATE DATABASE IF NOT EXISTS {} PRECISION 'ns'",
                self.login.get_database()
            ))
            .await?;
        self.client.use_database(self.login.get_database()).await?;
        Ok(())
    }
    async fn create_supertable(&self) -> Result<(), error::Error> {
        let metrics_string = format!(
            "ts TIMESTAMP, frametime TIMESTAMP{0}",
            (0..self.frame_data.num_channels)
                .map(|ch| format!(", c{ch} SMALLINT UNSIGNED"))
                .fold(String::new(), |a, b| a + &b)
        );
        let string = format!("CREATE STABLE IF NOT EXISTS template ({metrics_string}) TAGS (digitizer_id TINYINT UNSIGNED)");
        self.client
            .exec(&string)
            .await
            .map_err(|e| TDEngineError::SQL(SQLError::CreateTemplateTable, string.clone(), e))?;

        let frame_metrics_string = format!("frame_ts TIMESTAMP, sample_count INT UNSIGNED, sampling_rate INT UNSIGNED, frame_number INT UNSIGNED, error_code INT UNSIGNED{0}",
            (0..self.frame_data.num_channels)
                .map(|ch|format!(", cid{ch} INT UNSIGNED"))
                .fold(String::new(),|a,b|a + &b)
        );
        let string = format!("CREATE STABLE IF NOT EXISTS frame_template ({frame_metrics_string}) TAGS (digitizer_id TINYINT UNSIGNED)");
        self.client
            .exec(&string)
            .await
            .map_err(|e| TDEngineError::SQL(SQLError::CreateTemplateTable, string.clone(), e))?;
        Ok(())
    }
    pub async fn init_with_channel_count(
        &mut self,
        num_channels: usize,
    ) -> Result<(), error::Error> {
        self.frame_data.set_channel_count(num_channels);
        self.create_supertable().await?;

        //let stmt_sql = format!("INSERT INTO ? USING template TAGS (?, ?, ?{0}, ?) VALUES (?{0})", ", ?".repeat(num_channels));
        let stmt_sql = format!(
            "INSERT INTO ? USING template TAGS (?) VALUES (?, ?{0})",
            ", ?".repeat(num_channels)
        );
        self.stmt
            .prepare(&stmt_sql)
            .await
            .map_err(|e| TDEngineError::Stmt(StatementError::Prepare, e))?;

        let frame_stmt_sql = format!(
            "INSERT INTO ? USING frame_template TAGS (?) VALUES (?, ?, ?, ?, ?{0})",
            ", ?".repeat(num_channels)
        );
        self.frame_stmt
            .prepare(&frame_stmt_sql)
            .await
            .map_err(|e| TDEngineError::Stmt(StatementError::Prepare, e))?;
        Ok(())
    }

    pub async fn use_database(&mut self, database: &str) -> Result<()> {
        self.client.use_database(database).await?;
        Ok(())
    }

    pub async fn exec(&mut self, sql: &str) -> Result<usize, RawError> {
        self.client.exec(sql).await
    }

    pub async fn query(&mut self, sql: &str) -> Result<ResultSet, RawError> {
        self.client.query(sql).await
    }

    pub fn get_error_reporter(&mut self) -> &mut TDEngineErrorReporter {
        &mut self.error
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
    async fn process_message(
        &mut self,
        message: &DigitizerAnalogTraceMessage,
    ) -> Result<(), error::Error> {
        // Obtain the channel data, and error check
        self.error.test_metadata(message);

        // Obtain message data, and error check
        self.frame_data.init(message)?;

        // Obtain the channel data, and error check
        self.error
            .test_channels(&self.frame_data, &message.channels().unwrap());

        let table_name = self.frame_data.get_table_name();
        let frame_table_name = self.frame_data.get_frame_table_name();
        let channels = message.channels().ok_or(error::TraceMessageError::Frame(
            error::FrameError::ChannelsMissing,
        ))?;
        let frame_column_views =
            views::create_frame_column_views(&self.frame_data, &self.error, &channels)?;
        let column_views = views::create_column_views(&self.frame_data, &channels)?;
        let tags = [Value::UTinyInt(self.frame_data.digitizer_id)];

        //  Initialise Statement
        self.stmt
            .set_tbname(&table_name)
            .await
            .map_err(|e| TDEngineError::Stmt(StatementError::SetTableName, e))?;
        self.stmt
            .set_tags(&tags)
            .await
            .map_err(|e| TDEngineError::Stmt(StatementError::SetTags, e))?;
        self.stmt
            .bind(&column_views)
            .await
            .map_err(|e| TDEngineError::Stmt(StatementError::Bind, e))?;
        self.stmt
            .add_batch()
            .await
            .map_err(|e| TDEngineError::Stmt(StatementError::AddBatch, e))?;

        self.frame_stmt
            .set_tbname(&frame_table_name)
            .await
            .map_err(|e| TDEngineError::Stmt(StatementError::SetTableName, e))?;
        self.frame_stmt
            .set_tags(&tags)
            .await
            .map_err(|e| TDEngineError::Stmt(StatementError::SetTags, e))?;
        self.frame_stmt
            .bind(&frame_column_views)
            .await
            .map_err(|e| TDEngineError::Stmt(StatementError::Bind, e))?;
        self.frame_stmt
            .add_batch()
            .await
            .map_err(|e| TDEngineError::Stmt(StatementError::AddBatch, e))?;
        Ok(())
    }

    /// Sends data extracted from a previous call to ``process_message`` to the tdengine server.
    /// #Returns
    /// The number of rows affected by the post or an error
    async fn post_message(&mut self) -> Result<usize, error::Error> {
        Ok(self
            .stmt
            .execute()
            .await
            .map_err(|e| error::Error::TDEngine(TDEngineError::Stmt(StatementError::Execute, e)))?
            + self.frame_stmt.execute().await.map_err(|e| {
                error::Error::TDEngine(TDEngineError::Stmt(StatementError::Execute, e))
            })?)
    }
}
