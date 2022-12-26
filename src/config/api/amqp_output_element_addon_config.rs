use crate::config::api::amqp_queue_config::AmqpQueueConfig;
use lapin::options::BasicPublishOptions;
use lapin::BasicProperties;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone)]
pub struct AmqpOutputElementAddonConfig {
    queue: AmqpQueueConfig,
    channel_publish_exchange: String,
    channel_publish_options: BasicPublishOptions,
    channel_publish_properties: BasicProperties,
}

impl AmqpOutputElementAddonConfig {
    pub fn queue(&self) -> &AmqpQueueConfig {
        &self.queue
    }

    pub fn channel_publish_exchange(&self) -> &str {
        self.channel_publish_exchange.as_str()
    }

    pub fn channel_publish_options(&self) -> &BasicPublishOptions {
        &self.channel_publish_options
    }

    pub fn channel_publish_properties(&self) -> &BasicProperties {
        &self.channel_publish_properties
    }
}
