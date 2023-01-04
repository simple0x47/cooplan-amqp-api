use crate::api::input::input_element::InputElement;
use crate::error::Error;
use async_channel::Sender;
use cooplan_lapin_wrapper::config::amqp_connect_config::AmqpConnectConfig;
use cooplan_lapin_wrapper::config::api::Api;
use serde_json::Value;

use super::output::amqp_output_element::AmqpOutputElement;

pub type InputRegistration<LogicRequestType> =
    Box<dyn FnOnce(&Api) -> Result<Vec<InputElement<LogicRequestType>>, Error> + Send + Sync>;
pub type OutputRegistration =
    Box<dyn FnOnce(&Api) -> Result<Vec<AmqpOutputElement>, Error> + Send + Sync>;

pub struct InitializationPackage<LogicRequestType> {
    logic_request_sender: Sender<LogicRequestType>,
    pub input_registration: InputRegistration<LogicRequestType>,
    pub output_receiver: tokio::sync::mpsc::Receiver<(String, Value)>,
    pub output_registration: OutputRegistration,
    pub amqp_connect_config: AmqpConnectConfig,
}

impl<LogicRequestType> InitializationPackage<LogicRequestType> {
    pub fn new(
        logic_request_sender: Sender<LogicRequestType>,
        input_registration: InputRegistration<LogicRequestType>,
        output_receiver: tokio::sync::mpsc::Receiver<(String, Value)>,
        output_registration: OutputRegistration,
        amqp_connect_config: AmqpConnectConfig,
    ) -> InitializationPackage<LogicRequestType> {
        InitializationPackage {
            logic_request_sender,
            input_registration,
            output_receiver,
            output_registration,
            amqp_connect_config,
        }
    }

    pub fn logic_request_sender(&self) -> Sender<LogicRequestType> {
        self.logic_request_sender.clone()
    }
}
