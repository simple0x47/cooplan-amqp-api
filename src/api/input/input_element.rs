use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use crate::api::input::request::Request;
use crate::error::{Error, ErrorKind};
use async_channel::Sender;
use cooplan_amqp_api_shared::api::input::request_result::RequestResult;
use cooplan_lapin_wrapper::config::amqp_input_api::AmqpInputApi;
use cooplan_lapin_wrapper::config::api::Api;

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
    fn new(
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

pub fn extract_input<LogicRequestType>(
    api: &Api,
    id: &str,
    request_handler: RequestHandler<LogicRequestType>,
    actions: &'static [&'static str],
) -> Result<InputElement<LogicRequestType>, Error> {
    let api_config = match api.input().iter().find(|api_config| api_config.id() == id) {
        Some(api_config) => api_config,
        None => {
            return Err(Error::new(
                ErrorKind::AutoConfigFailure,
                format!("failed to find input api with id '{}'", id),
            ))
        }
    };

    Ok(InputElement::new(
        id.to_string(),
        request_handler,
        actions,
        api_config.clone(),
    ))
}
