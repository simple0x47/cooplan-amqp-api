use std::sync::atomic::AtomicU16;
use std::sync::Arc;

use crate::api::input::amqp_request_replier;
use crate::api::input::authorizer::Authorizer;
use async_channel::Sender;
use cooplan_amqp_api_shared::api::input::request_result::RequestResult;
use cooplan_state_tracker::state::State;
use cooplan_state_tracker::state_tracker_client::StateTrackerClient;
use futures_util::TryStreamExt;
use lapin::message::Delivery;
use lapin::{Channel, Consumer};
use serde_json::{Map, Value};
use uuid::Uuid;

use crate::api::input::input_element::InputElement;
use crate::api::input::request::Request;
use crate::api::input::sanitizer::sanitize;
use crate::error::{Error, ErrorKind};

use super::amqp_request_replier::AmqpRequestReplier;

pub struct AmqpRequestDispatch<LogicRequestType> {
    channel: Arc<Channel>,
    element: InputElement<LogicRequestType>,
    authorizer: Arc<Authorizer>,
    logic_request_sender: Sender<LogicRequestType>,
    current_concurrent_requests: Arc<AtomicU16>,
    state_tracker_client: StateTrackerClient,
}

impl<LogicRequestType: Send + 'static> AmqpRequestDispatch<LogicRequestType> {
    pub fn new(
        channel: Arc<Channel>,
        element: InputElement<LogicRequestType>,
        authorizer: Arc<Authorizer>,
        logic_request_sender: Sender<LogicRequestType>,
        mut state_tracker_client: StateTrackerClient,
    ) -> AmqpRequestDispatch<LogicRequestType> {
        state_tracker_client.set_id(element.name().to_string());

        AmqpRequestDispatch {
            channel,
            element,
            authorizer,
            logic_request_sender,
            current_concurrent_requests: Arc::new(AtomicU16::new(0)),
            state_tracker_client
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
                *self.element.config().queue_consumer().queue().declare().options(),
                self.element
                    .config()
                    .queue_consumer()
                    .queue()
                    .declare()
                    .arguments()
                    .clone(),
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
                self.element.config().queue_consumer().qos().prefetch_count(),
                *self.element.config().queue_consumer().qos().options(),
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

        let reject_options = *self.element.config().queue_consumer().reject();
        let acknowledge_options = *self.element.config().queue_consumer().acknowledge();
        let max_concurrent_requests = self.element.config().max_concurrent_requests();

        loop {
            if self
                .current_concurrent_requests
                .load(std::sync::atomic::Ordering::Relaxed)
                >= max_concurrent_requests
            {
                continue;
            }

            let state_tracker_client = self.state_tracker_client.clone();

            let delivery = match consumer.try_next().await {
                Ok(optional_delivery) => match optional_delivery {
                    Some(delivery) => delivery,
                    None => {
                        log::info!("consumer got an empty delivery");
                        continue;
                    }
                },
                Err(error) => {
                    let error_message = format!("consumer got an error: {}", error);

                    match state_tracker_client.send_state(State::Error(error_message.clone())).await {
                        Ok(_) => (),
                        Err(error) => log::error!("failed to send error state: {}", error)
                    }

                    log::warn!("{}", error_message);
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
                let mut state = State::Valid;

                match &result {
                    RequestResult::Ok(_) => {
                        if let Err(error) = delivery.ack(acknowledge_options).await {
                            let error_message = format!("failed to acknowledge delivery: {}", error);
                            log::error!("{}", error_message);

                            state = State::Error(error_message)
                        }
                    }
                    RequestResult::Err(error) => {
                        log::info!("failed to handle request: {}", error);

                        match delivery.reject(reject_options).await {
                            Ok(_) => (),
                            Err(error) => {
                                let error_message = format!("failed to reject delivery: {}", error);
                                log::error!("{}", error_message);

                                state = State::Error(error_message)
                            }
                        }
                    }
                }

                match state_tracker_client.send_state(state).await {
                    Ok(_) => (),
                    Err(error) => log::warn!("failed to send state: {}", error)
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
                *self.element.config().queue_consumer().consume().options(),
                self.element
                    .config()
                    .queue_consumer()
                    .consume()
                    .arguments()
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
        let reject_options = *self.element.config().queue_consumer().reject();

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
