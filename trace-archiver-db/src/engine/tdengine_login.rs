use dotenv;
use crate::utils::{unwrap_string_or_env_var, unwrap_num_or_env_var};

pub(crate) struct TDEngineLogin {
    url : String,
    database : String,
    user : String,
    password : String,
}

impl TDEngineLogin {
    pub fn from_optional(url : &Option<String>, port : &Option<u32>, user : &Option<String>, password : &Option<String>, database : &Option<String>) -> Self {
        let url = format!("{0}:{1}",
            unwrap_string_or_env_var(url, "TDENGINE_URL"),
            unwrap_num_or_env_var(port,"TDENGINE_PORT"),
        );
        let user = unwrap_string_or_env_var(user,"TDENGINE_DATABASE");
        let password = unwrap_string_or_env_var(password,"TDENGINE_USER");
        let database = unwrap_string_or_env_var(database,"TDENGINE_PASSWORD");
        TDEngineLogin { url, user, password, database }
    }

    pub(super) fn get_url(&self) -> &str { &self.url }
    pub(super) fn get_database(&self) -> &str { &self.database }
}
