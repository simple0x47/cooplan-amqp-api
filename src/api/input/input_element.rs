use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use async_channel::Sender;
use cooplan_lapin_wrapper::config::amqp_input_api::AmqpInputApi;
use crate::api::input::request::Request;
use crate::api::input::request_result::RequestResult;

pub type RequestHandler<LogicRequestType> = Arc<
    dyn Fn(
            Request,
            Sender<LogicRequestType>,
        ) -> Pin<Box<dyn Future<Output = RequestResult> + Send + Sync>>
        + Send
        + Sync,
>;

pub struct InputElement<LogicRequestType> {
    name: String,
    request_handler: RequestHandler<LogicRequestType>,
    actions: &'static [&'static str],
    config: AmqpInputApi,
}

impl<LogicRequestType> InputElement<LogicRequestType> {
    pub fn new(
        name: String,
        request_handler: RequestHandler<LogicRequestType>,
        actions: &'static [&'static str],
        config: AmqpInputApi,
    ) -> InputElement<LogicRequestType> {
        InputElement {
            name,
            request_handler,
            actions,
            config,
        }
    }

    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    pub fn request_handler(&self) -> RequestHandler<LogicRequestType> {
        self.request_handler.clone()
    }

    pub fn actions(&self) -> &'static [&'static str] {
        self.actions
    }

    pub fn config(&self) -> &AmqpInputApi {
        &self.config
    }
}
