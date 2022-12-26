use lapin::options::{
    BasicAckOptions, BasicConsumeOptions, BasicQosOptions, BasicRejectOptions, QueueDeclareOptions,
};
use lapin::types::FieldTable;
use serde::{Deserialize, Serialize};
use crate::config::api::amqp_queue_config::AmqpQueueConfig;

#[derive(Deserialize, Serialize, Clone)]
pub struct AmqpInputElementAddonConfig {
    queue: AmqpQueueConfig,
    channel_qos_prefetch_count: u16,
    channel_qos_options: BasicQosOptions,
    channel_consume_options: BasicConsumeOptions,
    channel_consume_arguments: FieldTable,
    channel_acknowledge_options: BasicAckOptions,
    channel_reject_options: BasicRejectOptions,
}

impl AmqpInputElementAddonConfig {
    pub fn queue(&self) -> &AmqpQueueConfig {
        &self.queue
    }

    pub fn channel_qos_prefetch_count(&self) -> u16 {
        self.channel_qos_prefetch_count
    }

    pub fn channel_qos_options(&self) -> &BasicQosOptions {
        &self.channel_qos_options
    }

    pub fn channel_consume_options(&self) -> &BasicConsumeOptions {
        &self.channel_consume_options
    }

    pub fn channel_consume_arguments(&self) -> &FieldTable {
        &self.channel_consume_arguments
    }

    pub fn channel_acknowledge_options(&self) -> &BasicAckOptions {
        &self.channel_acknowledge_options
    }

    pub fn channel_reject_options(&self) -> &BasicRejectOptions {
        &self.channel_reject_options
    }
}
