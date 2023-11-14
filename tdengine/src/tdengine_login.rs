use crate::error::EVError;

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
        let url = format!(
            "taos+ws://{0}",//@{2}:{3}",
            //user.ok_or(EVError::NotFound("TDEngine User Name"))?,
            //password.ok_or(EVError::NotFound("TDEngine Password"))?,
            broker.ok_or(EVError::NotFound("TDEngine Broker"))?,
        );
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
