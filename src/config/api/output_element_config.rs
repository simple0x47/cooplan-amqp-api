use serde::{Deserialize, Serialize};

use super::amqp_output_element_addon_config::AmqpOutputElementAddonConfig;

#[derive(Deserialize, Serialize, Clone)]
pub struct OutputElementConfig {
    name: String,
    amqp: AmqpOutputElementAddonConfig,
}

impl OutputElementConfig {
    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    pub fn amqp(&self) -> &AmqpOutputElementAddonConfig {
        &self.amqp
    }
}
