use std::sync::Arc;

use lapin::Channel;
use serde_json::Value;
use tokio::sync::mpsc::Receiver;

use crate::config::api::output_element_config::OutputElementConfig;

pub struct AmqpOutputElement {
    name: String,
    output_config: OutputElementConfig,
}

impl AmqpOutputElement {
    pub fn new(name: String, output_config: OutputElementConfig) -> AmqpOutputElement {
        AmqpOutputElement {
            name,
            output_config,
        }
    }

    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    pub fn output_config(&self) -> &OutputElementConfig {
        &self.output_config
    }

    pub fn owned_output_config(self) -> OutputElementConfig {
        self.output_config
    }
}

impl AmqpOutputElement {
    pub async fn run(self, channel: Arc<Channel>, mut receiver: Receiver<Value>) {
        let queue = self.output_config.amqp().queue();

        match channel
            .queue_declare(
                queue.name(),
                *queue.declare_options(),
                queue.declare_arguments().clone(),
            )
            .await
        {
            Ok(_) => (),
            Err(error) => {
                log::error!(
                    "failed to declare queue for output element '{}': '{}'",
                    self.name,
                    error
                );

                return;
            }
        };

        loop {
            let data = match receiver.recv().await {
                Some(data) => data,
                None => {
                    log::info!("received none from receiver");
                    continue;
                }
            };

            let payload = match serde_json::to_vec(&data) {
                Ok(payload) => payload,
                Err(error) => {
                    log::error!("failed to serialize output data as bytes: {}", error);
                    continue;
                }
            };

            match channel
                .basic_publish(
                    self.output_config.amqp().channel_publish_exchange(),
                    queue.name(),
                    *self.output_config.amqp().channel_publish_options(),
                    payload.as_slice(),
                    self.output_config
                        .amqp()
                        .channel_publish_properties()
                        .clone(),
                )
                .await
            {
                Ok(_) => (),
                Err(error) => {
                    log::error!("failed to publish to queue: {}", error);
                    continue;
                }
            }
        }
    }
}
