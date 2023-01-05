use std::sync::Arc;
use cooplan_lapin_wrapper::config::amqp_output_api::AmqpOutputApi;
use cooplan_state_tracker::state::State;
use cooplan_state_tracker::state_tracker_client::StateTrackerClient;

use lapin::Channel;
use serde_json::Value;
use tokio::sync::mpsc::Receiver;

pub struct AmqpOutputElement {
    name: String,
    output_config: AmqpOutputApi,
    state_tracker: StateTrackerClient,
}

impl AmqpOutputElement {
    pub fn new(name: String, output_config: AmqpOutputApi, mut state_tracker: StateTrackerClient) -> AmqpOutputElement {
        state_tracker.set_id(name.clone());

        AmqpOutputElement {
            name,
            output_config,
            state_tracker,
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
                handle_error(format!("failed to declare queue for output element '{}': '{}'",
                                     self.name,
                                     error), &self.state_tracker).await;

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
                    handle_error(format!("failed to serialize output data as bytes: {}", error), &self.state_tracker).await;
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
                    handle_error(format!("failed to publish to queue: {}", error), &self.state_tracker).await;
                    continue;
                }
            }

            match self.state_tracker.send_state(State::Valid).await {
                Ok(_) => (),
                Err(error) => log::warn!("failed to send valid state to state tracker: {}", error),
            }
        }
    }
}

async fn handle_error(error_message: String, state_tracker: &StateTrackerClient) {
    log::error!("{}", error_message);

    match state_tracker.send_state(State::Error(error_message)).await {
        Ok(_) => (),
        Err(error) => {
            log::warn!("failed to send error state to state tracker: '{}'", error);
        }
    }
}