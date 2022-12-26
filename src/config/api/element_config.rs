use serde::{Deserialize, Serialize};
use crate::config::api::input_element_config::InputElementConfig;
use crate::config::api::output_element_config::OutputElementConfig;

#[derive(Deserialize, Serialize)]
pub enum ElementConfig {
    Input(InputElementConfig),
    Output(OutputElementConfig)
}