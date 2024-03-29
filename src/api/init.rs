use std::sync::Arc;
use cooplan_lapin_wrapper::amqp_wrapper::AmqpWrapper;

use crate::api::initialization_package::InitializationPackage;
use crate::api::input::amqp_request_dispatch::AmqpRequestDispatch;
use crate::api::input::authorizer::try_generate_authorizer;
use crate::error::{Error, ErrorKind};

use super::output::amqp_output_router::AmqpOutputRouter;

pub async fn initialize<LogicRequestType: Send + 'static>(
    package: InitializationPackage<LogicRequestType>,
) -> Result<(), Error> {
    let logic_request_sender = package.logic_request_sender();

    let api = package.api;

    let input_registration = package.input_registration;
    let input_elements = input_registration(&api)?;

    let config = package.config;
    let authorizer = Arc::new(try_generate_authorizer(config.openid_connect).await?);

    let connect_config = config.amqp_connect_config;
    let mut amqp_wrapper =
        match AmqpWrapper::try_new(connect_config) {
            Ok(amqp_wrapper) => amqp_wrapper,
            Err(error) => return Err(Error::new(ErrorKind::InternalFailure, format!("failed to initialize amqp wrapper: {}", error))),
        };

    let state_tracker_client = package.state_tracker_client;

    for input_element in input_elements {
        let channel = match amqp_wrapper.try_get_channel().await {
            Ok(channel) => channel,
            Err(error) => return Err(Error::new(ErrorKind::InternalFailure, format!("failed to get channel: {}", error))),
        };

        let dispatch =
            AmqpRequestDispatch::new(channel, input_element, authorizer.clone(), logic_request_sender.clone(), state_tracker_client.clone());

        tokio::spawn(dispatch.run());
    }

    let output_registration = package.output_registration;
    let output_elements = output_registration(&api, state_tracker_client.clone())?;
    let output_channel = match amqp_wrapper.try_get_channel().await {
        Ok(channel) => channel,
        Err(error) => return Err(Error::new(ErrorKind::InternalFailure, format!("failed to get channel: {}", error))),
    };

    let output_router = AmqpOutputRouter::new(
        output_channel,
        output_elements,
        package.output_receiver,
    );

    tokio::spawn(output_router.run());

    Ok(())
}
