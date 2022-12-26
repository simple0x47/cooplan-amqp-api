use lapin::options::QueueDeclareOptions;
use lapin::types::FieldTable;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone)]
pub struct AmqpQueueConfig {
    name: String,
    declare_options: QueueDeclareOptions,
    declare_arguments: FieldTable,
}

impl AmqpQueueConfig {
    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    pub fn declare_options(&self) -> &QueueDeclareOptions {
        &self.declare_options
    }

    pub fn declare_arguments(&self) -> &FieldTable {
        &self.declare_arguments
    }
}