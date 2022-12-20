use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use async_channel::Sender;

use crate::api::request::Request;
use crate::api::request_result::RequestResult;
use crate::config::api::element_config::ElementConfig;

pub type RequestHandler<LogicRequestType> = Arc<
    dyn Fn(
            Request,
            Sender<LogicRequestType>,
        ) -> Pin<Box<dyn Future<Output = RequestResult> + Send + Sync>>
        + Send
        + Sync,
>;

#[derive(Clone)]
pub struct Element<LogicRequestType> {
    name: String,
    request_handler: RequestHandler<LogicRequestType>,
    actions: &'static [&'static str],
    config: ElementConfig,
}

impl<LogicRequestType> Element<LogicRequestType> {
    pub fn new(
        name: String,
        request_handler: RequestHandler<LogicRequestType>,
        actions: &'static [&'static str],
        config: ElementConfig,
    ) -> Element<LogicRequestType> {
        Element {
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

    pub fn config(&self) -> &ElementConfig {
        &self.config
    }
}
