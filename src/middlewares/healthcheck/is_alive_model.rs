use std::env;
use serde::Serialize;

const APP_VERSION: &str = "APP_VERSION";
const DATE_COMPILATION: &str = "DATE_COMPILATION";
const ENV_INFO: &str = "ENV_INFO";
const SESSION_ENCODING_KEY: &str = "SESSION_ENCODING_KEY";

#[derive(Serialize)]
pub struct SessionId {
    #[serde(rename = "SESSION_ENCODING_KEY")]
    pub key: String
}

#[derive(Serialize)]
pub struct IsAliveModel {
    #[serde(rename = "IsAlive")]
    pub is_alive: bool,

    #[serde(rename = "FrameworkVersion")]
    pub framework_version: String, 
    
    #[serde(rename = "AppVersion")]
    pub app_version: String,
    
    #[serde(rename = "AppCompilationDate")]
    pub app_compilation_date: String,
    
    #[serde(rename = "EnvInfo")]
    pub env_info: String,
    
    #[serde(rename = "EnvVariablesSha1")]
    pub env_variables_sha1: SessionId
}

fn read_env_var(env_name: &str) -> String {
    return match env::var(env_name) {
        Ok(v) => v,
        _ => String::new(),
    }; 
}

pub fn read() -> IsAliveModel {
    return IsAliveModel {
        is_alive: true,
        framework_version: read_env_var("CARGO_PKG_VERSION"),
        app_version: read_env_var(APP_VERSION),
        app_compilation_date: read_env_var(DATE_COMPILATION),
        env_info: read_env_var(ENV_INFO),
        env_variables_sha1: SessionId { key: read_env_var(SESSION_ENCODING_KEY) },
    }   
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_isalive_model() {
        // decalre env values 
        let app_version = "registry/youhodler-api:1.0.65-test-320274105";
        let app_compilation_date = "2021-06-14T07:33:08+00:00";
        let env_info = "api-7d5874b885-htrxn";
        let encoding_key = "F9341D7F4C91DDF7E7E1E4CF57E40D29EA10CB06";

        // set env variables 
        env::set_var(APP_VERSION, app_version);
        env::set_var(DATE_COMPILATION, app_compilation_date);
        env::set_var(ENV_INFO, env_info);
        env::set_var(SESSION_ENCODING_KEY, encoding_key);

        // act
        let data = read();

        // assert
        assert_eq!(data.is_alive, true);
        assert_eq!(data.framework_version, String::from(env!("CARGO_PKG_VERSION")));    
        assert_eq!(data.app_compilation_date, app_compilation_date);     
        assert_eq!(data.env_info, env_info);
        assert_eq!(data.env_variables_sha1.key, encoding_key);
    }
}