use std::sync::Arc;

use crate::api::amqp_connection_manager::AmqpConnectionManager;
use crate::api::amqp_request_dispatch::AmqpRequestDispatch;
use crate::api::authorizer::try_generate_authorizer;
use crate::api::initialization_package::InitializationPackage;
use crate::config::api::amqp_connection_manager_config;
use crate::error::Error;

pub async fn initialize<LogicRequestType: Send + 'static>(
    package: InitializationPackage<LogicRequestType>,
) -> Result<(), Error> {
    let sender = package.sender();
    let config = crate::config::api::config::try_generate_config()?;

    let elements = package.registration()(&config)?;
    let authorizer = Arc::new(try_generate_authorizer().await?);

    let amqp_connection_manager_config = amqp_connection_manager_config::try_generate_config()?;
    let amqp_connection_manager =
        AmqpConnectionManager::try_new(amqp_connection_manager_config).await?;

    for element in elements {
        let channel = amqp_connection_manager.try_get_channel().await?;
        let dispatch =
            AmqpRequestDispatch::new(channel, element, authorizer.clone(), sender.clone());

        tokio::spawn(dispatch.run());
    }

    Ok(())
}
