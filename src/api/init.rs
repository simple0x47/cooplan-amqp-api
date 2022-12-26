use std::sync::Arc;

use crate::api::amqp_connection_manager::AmqpConnectionManager;
use crate::api::initialization_package::InitializationPackage;
use crate::api::input::amqp_request_dispatch::AmqpRequestDispatch;
use crate::api::input::authorizer::try_generate_authorizer;
use crate::config::api::amqp_connection_manager_config;
use crate::error::Error;

use super::output::amqp_output_router::AmqpOutputRouter;

pub async fn initialize<LogicRequestType: Send + 'static>(
    package: InitializationPackage<LogicRequestType>,
) -> Result<(), Error> {
    let sender = package.sender();
    let config = crate::config::api::config::try_generate_config()?;

    let input_registration = package.input_registration;
    let input_elements = input_registration(&config)?;

    let authorizer = Arc::new(try_generate_authorizer().await?);

    let amqp_connection_manager_config = amqp_connection_manager_config::try_generate_config()?;
    let amqp_connection_manager =
        AmqpConnectionManager::try_new(amqp_connection_manager_config).await?;

    for input_element in input_elements {
        let channel = amqp_connection_manager.try_get_channel().await?;
        let dispatch =
            AmqpRequestDispatch::new(channel, input_element, authorizer.clone(), sender.clone());

        tokio::spawn(dispatch.run());
    }

    let output_registration = package.output_registration;
    let output_elements = output_registration(&config)?;

    let output_router = AmqpOutputRouter::new(
        amqp_connection_manager.try_get_channel().await?,
        output_elements,
        package.output_receiver,
    );

    tokio::spawn(output_router.run());

    Ok(())
}
