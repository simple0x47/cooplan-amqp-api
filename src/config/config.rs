use serde::{Deserialize, Serialize};

use crate::config::api::openid_connect_config::OpenIdConnectConfig;
use crate::error::{Error, ErrorKind};

const CONFIG_FILE: &str = "./config.json";

#[derive(Serialize, Deserialize)]
pub struct Config {
    openid_connect: OpenIdConnectConfig,
}

impl Config {
    pub fn openid_connect(&self) -> &OpenIdConnectConfig {
        &self.openid_connect
    }

    pub fn owned_openid_connect(self) -> OpenIdConnectConfig {
        self.openid_connect
    }
}

pub async fn try_read_config() -> Result<Config, Error> {
    let config = match tokio::fs::read_to_string(CONFIG_FILE).await {
        Ok(config) => match serde_json::from_str::<Config>(config.as_str()) {
            Ok(config) => config,
            Err(error) => {
                return Err(Error::new(
                    ErrorKind::AutoConfigFailure,
                    format!("failed to deserialize config file's content: {}", error),
                ));
            }
        },
        Err(error) => {
            return Err(Error::new(
                ErrorKind::AutoConfigFailure,
                format!("failed to read config file: {}", error),
            ));
        }
    };

    Ok(config)
}
