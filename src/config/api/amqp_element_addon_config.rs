use lapin::options::{
    BasicAckOptions, BasicConsumeOptions, BasicQosOptions, BasicRejectOptions, QueueDeclareOptions,
};
use lapin::types::FieldTable;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone)]
pub struct AmqpElementAddonConfig {
    queue_declare_options: QueueDeclareOptions,
    queue_declare_arguments: FieldTable,

    channel_qos_prefetch_count: u16,
    channel_qos_options: BasicQosOptions,
    channel_consume_options: BasicConsumeOptions,
    channel_consume_arguments: FieldTable,
    channel_acknowledge_options: BasicAckOptions,
    channel_reject_options: BasicRejectOptions,
}

impl AmqpElementAddonConfig {
    pub fn queue_options(&self) -> &QueueDeclareOptions {
        &self.queue_declare_options
    }

    pub fn queue_arguments(&self) -> &FieldTable {
        &self.queue_declare_arguments
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
