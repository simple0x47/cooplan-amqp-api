use cooplan_lapin_wrapper::config::amqp_connect_config::AmqpConnectConfig;
use serde::{Deserialize};

use crate::config::openid_connect_config::OpenIdConnectConfig;
use crate::error::{Error, ErrorKind};

#[derive(Deserialize)]
pub struct Config {
    pub openid_connect: OpenIdConnectConfig,
    pub amqp_connect_config: AmqpConnectConfig
}

pub async fn try_read_config(config_file: &str) -> Result<Config, Error> {
    let config = match tokio::fs::read_to_string(config_file).await {
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
