use std::sync::Arc;
use cooplan_lapin_wrapper::config::amqp_output_api::AmqpOutputApi;

use lapin::Channel;
use serde_json::Value;
use tokio::sync::mpsc::Receiver;

pub struct AmqpOutputElement {
    name: String,
    output_config: AmqpOutputApi,
}

impl AmqpOutputElement {
    pub fn new(name: String, output_config: AmqpOutputApi) -> AmqpOutputElement {
        AmqpOutputElement {
            name,
            output_config,
        }
    }

    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    pub fn output_config(&self) -> &AmqpOutputApi {
        &self.output_config
    }

    pub fn owned_output_config(self) -> AmqpOutputApi {
        self.output_config
    }
}

impl AmqpOutputElement {
    pub async fn run(self, channel: Arc<Channel>, mut receiver: Receiver<Value>) {
        let queue = self.output_config.queue();

        match channel
            .queue_declare(
                queue.name(),
                *queue.declare().options(),
                queue.declare().arguments().clone(),
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
                    self.output_config.publish().exchange(),
                    queue.name(),
                    *self.output_config.publish().options(),
                    payload.as_slice(),
                    self.output_config
                        .publish()
                        .properties()
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
