use async_channel::Sender;
use crate::api::input::element::Element;
use crate::config::api::config::Config;
use crate::error::Error;

pub type Registration<LogicRequestType> = Box<dyn FnOnce(&Config) -> Result<Vec<Element<LogicRequestType>>, Error> + Send + Sync>;

pub struct InitializationPackage<LogicRequestType> {
    sender: Sender<LogicRequestType>,
    registration: Registration<LogicRequestType>,
}

impl<LogicRequestType> InitializationPackage<LogicRequestType> {
    pub fn new(sender: Sender<LogicRequestType>, registration: Registration<LogicRequestType>) -> InitializationPackage<LogicRequestType> {
        InitializationPackage {
            sender,
            registration
        }
    }

    pub fn sender(&self) -> Sender<LogicRequestType> {
        self.sender.clone()
    }

    pub fn registration(self) -> Registration<LogicRequestType> {
        self.registration
    }
}