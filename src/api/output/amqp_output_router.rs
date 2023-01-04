use std::{collections::HashMap, sync::Arc};

use lapin::Channel;
use serde_json::Value;
use tokio::sync::mpsc::{Receiver, Sender};
use crate::api::output::amqp_output_element::AmqpOutputElement;

pub struct AmqpOutputRouter {
    receiver: Receiver<(String, Value)>,
    output_senders: HashMap<String, Sender<Value>>,
}

impl AmqpOutputRouter {
    pub fn new(
        channel: Arc<Channel>,
        elements: Vec<AmqpOutputElement>,
        receiver: Receiver<(String, Value)>,
    ) -> AmqpOutputRouter {
        let mut output_senders = HashMap::new();
        let channel = channel;

        for element in elements {
            let (sender, receiver) = tokio::sync::mpsc::channel(1024);
            output_senders.insert(element.name().to_string(), sender);
            tokio::spawn(element.run(channel.clone(), receiver));
        }

        AmqpOutputRouter {
            receiver,
            output_senders,
        }
    }

    pub async fn run(mut self) {
        loop {
            let element_and_data = match self.receiver.recv().await {
                Some(delivery) => delivery,
                None => {
                    log::info!("received none from receiver");
                    continue;
                }
            };

            let output_sender = match self.output_senders.get(&element_and_data.0) {
                Some(config) => config,
                None => {
                    log::error!("missing output element for '{}'", element_and_data.0);
                    continue;
                }
            };

            match output_sender.send(element_and_data.1).await {
                Ok(_) => (),
                Err(error) => {
                    log::error!(
                        "failed to send data to output element '{}': '{}'",
                        element_and_data.0,
                        error
                    );
                }
            }
        }
    }
}
