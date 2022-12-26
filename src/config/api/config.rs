use serde::{Deserialize, Serialize};

use crate::error::{Error, ErrorKind};

use super::{input_element_config::InputElementConfig, output_element_config::OutputElementConfig};

#[derive(Deserialize, Serialize)]
pub struct Config {
    input: Vec<InputElementConfig>,
    output: Vec<OutputElementConfig>,
}

impl Config {
    pub fn input_element_configs(&self) -> &[InputElementConfig] {
        self.input.as_slice()
    }

    pub fn try_get_input_element_config(&self, name: &str) -> Result<InputElementConfig, Error> {
        for input_element in self.input.as_slice() {
            if input_element.name().eq(name) {
                return Ok(input_element.clone());
            }
        }

        Err(Error::new(
            ErrorKind::InternalFailure,
            format!("input element '{}' could not be found", name),
        ))
    }

    pub fn output_element_configs(&self) -> &[OutputElementConfig] {
        self.output.as_slice()
    }

    pub fn try_get_output_element_config(&self, name: &str) -> Result<OutputElementConfig, Error> {
        for output_element in self.output.as_slice() {
            if output_element.name().eq(name) {
                return Ok(output_element.clone());
            }
        }

        Err(Error::new(
            ErrorKind::InternalFailure,
            format!("output element '{}' could not be found", name),
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

    let config = match serde_json::from_str::<Config>(api_string.as_str()) {
        Ok(config) => config,
        Err(error) => {
            return Err(Error::new(
                ErrorKind::AutoConfigFailure,
                format!("failed to deserialize input elements: {}", error),
            ));
        }
    };

    Ok(config)
}
