use crate::utils::{unwrap_num_or_env_var, unwrap_string_or_env_var};
use dotenv;

#[derive(Debug)]
pub(crate) struct TDEngineLogin {
    url: String,
    database: String,
}

impl TDEngineLogin {
    pub fn from_optional(
        url: Option<String>,
        port: Option<u32>,
        user: Option<String>,
        password: Option<String>,
        database: Option<String>,
    ) -> Self {
        let url = format!(
            "taos://{0}:{1}@{2}:{3}",
            unwrap_string_or_env_var(user, "TDENGINE_USER"),
            unwrap_string_or_env_var(password, "TDENGINE_PASSWORD"),
            unwrap_string_or_env_var(url, "TDENGINE_URL"),
            unwrap_num_or_env_var(port, "TDENGINE_PORT"),
        );
        let database = unwrap_string_or_env_var(database, "TDENGINE_DATABASE");
        TDEngineLogin { url, database }
    }

    pub(super) fn get_url(&self) -> &str {
        &self.url
    }
    pub(super) fn get_database(&self) -> &str {
        &self.database
    }
}
