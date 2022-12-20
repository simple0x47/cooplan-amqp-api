use std::sync::Arc;

use lapin::message::Delivery;
use lapin::options::BasicPublishOptions;
use lapin::types::ShortString;
use lapin::{BasicProperties, Channel};

use crate::api::request_result::RequestResult;
use crate::error::{Error, ErrorKind};

pub struct AmqpRequestReplier<'reply> {
    channel: &'reply Arc<Channel>,
    reply_to: &'reply str,
    response_properties: BasicProperties,
}

impl<'reply> AmqpRequestReplier<'reply> {
    fn new(
        channel: &'reply Arc<Channel>,
        reply_to: &'reply str,
        response_properties: BasicProperties,
    ) -> AmqpRequestReplier<'reply> {
        AmqpRequestReplier {
            channel,
            reply_to,
            response_properties,
        }
    }

    pub async fn reply(&'reply self, result: RequestResult) -> Result<(), Error> {
        let options = BasicPublishOptions::default();
        let payload = match serde_json::to_vec(&result) {
            Ok(payload) => payload,
            Err(error) => {
                return Err(Error::new(
                    ErrorKind::InternalFailure,
                    format!("failed to serialize result: {}", error),
                ));
            }
        };

        match self
            .channel
            .basic_publish(
                "",
                self.reply_to,
                options,
                payload.as_slice(),
                self.response_properties.clone(),
            )
            .await
        {
            Ok(_) => Ok(()),
            Err(error) => Err(Error::new(
                ErrorKind::AmqpFailure,
                format!("failed to send reply: {}", error),
            )),
        }
    }
}

pub fn try_generate_replier<'reply>(
    channel: &'reply Arc<Channel>,
    delivery: &'reply Delivery,
) -> Option<AmqpRequestReplier<'reply>> {
    let request_properties = &delivery.properties;

    let reply_to = match request_properties.reply_to() {
        Some(reply_to) => reply_to,
        None => return None,
    };

    let mut properties =
        BasicProperties::default().with_content_type(ShortString::from("application/json"));

    if let Some(correlation_id) = request_properties.correlation_id() {
        properties = properties.with_correlation_id(correlation_id.clone());
    }

    Some(AmqpRequestReplier::new(
        channel,
        reply_to.as_str(),
        properties,
    ))
}
