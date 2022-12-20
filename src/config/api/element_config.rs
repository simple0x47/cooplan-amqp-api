use serde::{Deserialize, Serialize};

use crate::config::api::amqp_element_addon_config::AmqpElementAddonConfig;

#[derive(Deserialize, Serialize, Clone)]
pub struct ElementConfig {
    name: String,
    amqp: AmqpElementAddonConfig,
    max_concurrent_requests: u16,
}

impl ElementConfig {
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn amqp(&self) -> &AmqpElementAddonConfig {
        &self.amqp
    }
    pub fn max_concurrent_requests(&self) -> u16 {
        self.max_concurrent_requests
    }
}
