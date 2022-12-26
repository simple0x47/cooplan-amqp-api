use crate::api::input::input_element::InputElement;
use crate::config::api::config::Config;
use crate::error::Error;
use async_channel::Sender;
use serde_json::Value;

use super::output::amqp_output_element::AmqpOutputElement;

pub type InputRegistration<LogicRequestType> =
    Box<dyn FnOnce(&Config) -> Result<Vec<InputElement<LogicRequestType>>, Error> + Send + Sync>;
pub type OutputRegistration =
    Box<dyn FnOnce(&Config) -> Result<Vec<AmqpOutputElement>, Error> + Send + Sync>;

pub struct InitializationPackage<LogicRequestType> {
    sender: Sender<LogicRequestType>,
    pub input_registration: InputRegistration<LogicRequestType>,
    pub output_receiver: tokio::sync::mpsc::Receiver<(String, Value)>,
    pub output_registration: OutputRegistration,
}

impl<LogicRequestType> InitializationPackage<LogicRequestType> {
    pub fn new(
        sender: Sender<LogicRequestType>,
        input_registration: InputRegistration<LogicRequestType>,
        output_receiver: tokio::sync::mpsc::Receiver<(String, Value)>,
        output_registration: OutputRegistration,
    ) -> InitializationPackage<LogicRequestType> {
        InitializationPackage {
            sender,
            input_registration,
            output_receiver,
            output_registration,
        }
    }

    pub fn sender(&self) -> Sender<LogicRequestType> {
        self.sender.clone()
    }
}
