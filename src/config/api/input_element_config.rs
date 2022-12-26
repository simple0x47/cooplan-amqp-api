use serde::{Deserialize, Serialize};

use crate::config::api::amqp_input_element_addon_config::AmqpInputElementAddonConfig;

#[derive(Deserialize, Serialize, Clone)]
pub struct InputElementConfig {
    name: String,
    amqp: AmqpInputElementAddonConfig,
    max_concurrent_requests: u16,
}

impl InputElementConfig {
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn amqp(&self) -> &AmqpInputElementAddonConfig {
        &self.amqp
    }
    pub fn max_concurrent_requests(&self) -> u16 {
        self.max_concurrent_requests
    }
}
