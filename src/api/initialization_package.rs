use crate::api::input::input_element::InputElement;
use crate::error::Error;
use async_channel::Sender;
use cooplan_lapin_wrapper::config::api::Api;
use cooplan_state_tracker::state_tracker_client::StateTrackerClient;
use serde_json::Value;
use crate::config::config::Config;

use super::output::amqp_output_element::AmqpOutputElement;

pub type InputRegistration<LogicRequestType> =
    Box<dyn FnOnce(&Api) -> Result<Vec<InputElement<LogicRequestType>>, Error> + Send + Sync>;
pub type OutputRegistration =
    Box<dyn FnOnce(&Api, StateTrackerClient) -> Result<Vec<AmqpOutputElement>, Error> + Send + Sync>;

pub struct InitializationPackage<LogicRequestType> {
    pub logic_request_sender: Sender<LogicRequestType>,
    pub input_registration: InputRegistration<LogicRequestType>,
    pub output_receiver: tokio::sync::mpsc::Receiver<(String, Value)>,
    pub output_registration: OutputRegistration,
    pub api: Api,
    pub config: Config,
    pub state_tracker_client: StateTrackerClient
}

impl<LogicRequestType> InitializationPackage<LogicRequestType> {
    pub fn new(
        logic_request_sender: Sender<LogicRequestType>,
        input_registration: InputRegistration<LogicRequestType>,
        output_receiver: tokio::sync::mpsc::Receiver<(String, Value)>,
        output_registration: OutputRegistration,
        api: Api,
        config: Config,
        state_tracker_client: StateTrackerClient
    ) -> InitializationPackage<LogicRequestType> {
        InitializationPackage {
            logic_request_sender,
            input_registration,
            output_receiver,
            output_registration,
            api,
            config,
            state_tracker_client
        }
    }

    pub fn logic_request_sender(&self) -> Sender<LogicRequestType> {
        self.logic_request_sender.clone()
    }
}
