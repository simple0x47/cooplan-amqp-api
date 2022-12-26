use crate::error::{Error, ErrorKind};

use super::input_element_config::InputElementConfig;

pub struct Config {
    elements: Vec<InputElementConfig>,
}

impl Config {
    pub fn new(elements: Vec<InputElementConfig>) -> Config {
        Config { elements }
    }

    pub fn api_items(&self) -> &[InputElementConfig] {
        self.elements.as_slice()
    }

    pub fn try_get_api_item(&self, name: &str) -> Result<InputElementConfig, Error> {
        for api_item in self.elements.as_slice() {
            if api_item.name().eq(name) {
                return Ok(api_item.clone());
            }
        }

        Err(Error::new(
            ErrorKind::InternalFailure,
            format!("api item '{}' could not be found", name),
        ))
    }
}

pub const API_FILE: &str = "api.json";

pub fn try_generate_config() -> Result<Config, Error> {
    let api_string = match std::fs::read_to_string(API_FILE) {
        Ok(api_string) => api_string,
        Err(error) => {
            return Err(Error::new(
                ErrorKind::AutoConfigFailure,
                format!("failed to read {}: {}", API_FILE, error),
            ));
        }
    };

    let api_items = match serde_json::from_str::<Vec<InputElementConfig>>(api_string.as_str()) {
        Ok(api_items) => api_items,
        Err(error) => {
            return Err(Error::new(
                ErrorKind::AutoConfigFailure,
                format!("failed to deserialize api items: {}", error),
            ));
        }
    };

    Ok(Config::new(api_items))
}
