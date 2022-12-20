use std::sync::atomic::AtomicU16;
use std::sync::Arc;

use async_channel::Sender;
use futures_util::TryStreamExt;
use lapin::message::Delivery;
use lapin::{Channel, Consumer};
use serde_json::{Map, Value};
use uuid::Uuid;

use crate::api::amqp_request_replier;
use crate::api::authorizer::Authorizer;
use crate::api::element::Element;
use crate::api::request::Request;
use crate::api::request_result::RequestResult;
use crate::api::sanitizer::sanitize;
use crate::error::{Error, ErrorKind};

use super::amqp_request_replier::AmqpRequestReplier;

pub struct AmqpRequestDispatch<LogicRequestType> {
    channel: Arc<Channel>,
    element: Element<LogicRequestType>,
    authorizer: Arc<Authorizer>,
    logic_request_sender: Sender<LogicRequestType>,
    current_concurrent_requests: Arc<AtomicU16>,
}

impl<LogicRequestType: Send + 'static> AmqpRequestDispatch<LogicRequestType> {
    pub fn new(
        channel: Channel,
        element: Element<LogicRequestType>,
        authorizer: Arc<Authorizer>,
        logic_request_sender: Sender<LogicRequestType>,
    ) -> AmqpRequestDispatch<LogicRequestType> {
        AmqpRequestDispatch {
            channel: Arc::new(channel),
            element,
            authorizer,
            logic_request_sender,
            current_concurrent_requests: Arc::new(AtomicU16::new(0)),
        }
    }

    /// Blocks thread as long as the program is running.
    /// Deliveries are received, sanitized and authorized before being moved into a
    /// new task where the request will be handled.
    pub async fn run(self) -> Result<(), Error> {
        let queue = match self
            .channel
            .queue_declare(
                self.element.name(),
                *self.element.config().amqp().queue_options(),
                self.element.config().amqp().queue_arguments().clone(),
            )
            .await
        {
            Ok(queue) => queue,
            Err(error) => {
                return Err(Error::new(
                    ErrorKind::AmqpFailure,
                    format!("failed to declare queue: {}", error),
                ));
            }
        };

        match self
            .channel
            .basic_qos(
                self.element.config().amqp().channel_qos_prefetch_count(),
                *self.element.config().amqp().channel_qos_options(),
            )
            .await
        {
            Ok(()) => (),
            Err(error) => {
                return Err(Error::new(
                    ErrorKind::AmqpFailure,
                    format!("failure basic qos: {}", error),
                ));
            }
        }

        let mut consumer = self.try_get_consumer(queue.name().as_str()).await?;

        let reject_options = *self.element.config().amqp().channel_reject_options();
        let acknowledge_options = *self.element.config().amqp().channel_acknowledge_options();
        let max_concurrent_requests = self.element.config().max_concurrent_requests();

        loop {
            if self
                .current_concurrent_requests
                .load(std::sync::atomic::Ordering::Relaxed)
                >= max_concurrent_requests
            {
                continue;
            }

            let delivery = match consumer.try_next().await {
                Ok(optional_delivery) => match optional_delivery {
                    Some(delivery) => delivery,
                    None => {
                        log::info!("consumer got an empty delivery");
                        continue;
                    }
                },
                Err(error) => {
                    log::warn!("consumer got an error: {}", error);
                    continue;
                }
            };

            let channel = self.channel.clone();

            let request_replier: Option<AmqpRequestReplier> =
                amqp_request_replier::try_generate_replier(&channel, &delivery);

            let request = match self.prepare_request(&delivery).await {
                Ok(request) => request,
                Err(error) => {
                    if let Some(request_replier) = request_replier {
                        match request_replier
                            .reply(RequestResult::Err(error.clone().into()))
                            .await
                        {
                            Ok(_) => (),
                            Err(error) => {
                                log::warn!("failed to reply: {}", error);
                            }
                        }
                    }

                    log::info!("failed to prepare request: {}", error);
                    continue;
                }
            };

            let request_handler = self.element.request_handler();

            let logic_request_sender = self.logic_request_sender.clone();

            self.current_concurrent_requests
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed);

            let current_concurrent_requests = self.current_concurrent_requests.clone();
            tokio::spawn(async move {
                let result = request_handler(request, logic_request_sender).await;

                match &result {
                    RequestResult::Ok(_) => {
                        if let Err(error) = delivery.ack(acknowledge_options).await {
                            log::warn!("failed to acknowledge delivery: {}", error);
                        }
                    }
                    RequestResult::Err(error) => {
                        log::info!("failed to handle request: {}", error);

                        match delivery.reject(reject_options).await {
                            Ok(_) => (),
                            Err(error) => {
                                log::warn!("failed to reject delivery: {}", error);
                            }
                        }
                    }
                }

                if let Some(amqp_request_replier) =
                    amqp_request_replier::try_generate_replier(&channel, &delivery)
                {
                    match amqp_request_replier.reply(result).await {
                        Ok(_) => (),
                        Err(error) => {
                            log::info!("failed to reply: {}", error);
                        }
                    }
                }

                current_concurrent_requests.fetch_sub(1, std::sync::atomic::Ordering::Relaxed);
            });
        }
    }

    async fn try_get_consumer(&self, queue_name: &str) -> Result<Consumer, Error> {
        let consumer_tag = format!("{}#{}", queue_name, Uuid::new_v4());
        let consumer = match self
            .channel
            .basic_consume(
                queue_name,
                consumer_tag.as_str(),
                *self.element.config().amqp().channel_consume_options(),
                self.element
                    .config()
                    .amqp()
                    .channel_consume_arguments()
                    .clone(),
            )
            .await
        {
            Ok(consumer) => consumer,
            Err(error) => {
                return Err(Error::new(
                    ErrorKind::AmqpFailure,
                    format!("failure basic consume: {}", error),
                ));
            }
        };

        Ok(consumer)
    }

    async fn prepare_request(&self, delivery: &Delivery) -> Result<Request, Error> {
        let reject_options = *self.element.config().amqp().channel_reject_options();

        let request_data = match std::str::from_utf8(delivery.data.as_slice()) {
            Ok(request_data) => request_data,
            Err(error) => {
                return match delivery.reject(reject_options).await {
                    Ok(_) => Err(Error::new(
                        ErrorKind::MalformedRequest,
                        format!("delivery is not an utf8 string: {}", error),
                    )),
                    Err(error) => Err(Error::new(
                        ErrorKind::AmqpFailure,
                        format!("failed to reject delivery: {}", error),
                    )),
                };
            }
        };

        let raw_request = match serde_json::from_str::<Map<String, Value>>(request_data) {
            Ok(raw_request) => raw_request,
            Err(error) => {
                return match delivery.reject(reject_options).await {
                    Ok(()) => Err(Error::new(
                        ErrorKind::MalformedRequest,
                        format!("delivery is not a json object: {}", error),
                    )),
                    Err(error) => Err(Error::new(
                        ErrorKind::AmqpFailure,
                        format!("failed to reject delivery: {}", error),
                    )),
                };
            }
        };

        let mut request = match sanitize(raw_request, self.element.actions()) {
            Ok(request) => request,
            Err(error) => {
                return match delivery.reject(reject_options).await {
                    Ok(()) => Err(Error::new(
                        ErrorKind::MalformedRequest,
                        format!("request sanitization failure: {}", error),
                    )),
                    Err(error) => Err(Error::new(
                        ErrorKind::AmqpFailure,
                        format!("failed to reject delivery: {}", error),
                    )),
                };
            }
        };

        request = match self.authorizer.authorize(request) {
            Ok(request) => request,
            Err(error) => {
                return match delivery.reject(reject_options).await {
                    Ok(()) => Err(Error::new(
                        ErrorKind::MalformedRequest,
                        format!("request sanitization failure: {}", error),
                    )),
                    Err(error) => Err(Error::new(
                        ErrorKind::AmqpFailure,
                        format!("failed to reject delivery: {}", error),
                    )),
                };
            }
        };

        Ok(request)
    }
}