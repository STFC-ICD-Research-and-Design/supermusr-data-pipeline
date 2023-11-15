use super::error::EVError;

#[derive(Debug)]
pub(crate) struct TDEngineLogin {
    url: String,
    database: String,
}

impl TDEngineLogin {
    pub fn from_optional(
        broker: Option<String>,
        user: Option<String>,
        password: Option<String>,
        database: Option<String>,
    ) -> Result<Self, EVError> {
        let broker = broker.ok_or(EVError::NotFound("TDEngine Broker"))?;
        let url = Option::zip(user,password)
            .map(|(user,password)|format!("taos+ws://{broker}@{user}:{password}"))
            .unwrap_or_else(||format!("taos+ws://{broker}"));
        let database = database.ok_or(EVError::NotFound("TDEngine Database"))?;
        Ok(TDEngineLogin { url, database })
    }

    pub(super) fn get_url(&self) -> &str {
        &self.url
    }
    pub(super) fn get_database(&self) -> &str {
        &self.database
    }
}
