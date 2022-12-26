use serde::de::DeserializeOwned;
use serde_json::{Map, Value};
use crate::api::input::request_header::RequestHeader;

use crate::error::{Error, ErrorKind};

#[derive(Debug)]
pub struct Request {
    request: Map<String, Value>,
}

const HEADER_KEY: &str = "header";

impl Request {
    pub fn new(request: Map<String, Value>) -> Request {
        Request { request }
    }

    pub fn data(self) -> Map<String, Value> {
        self.request
    }

    pub fn try_get_token(&self) -> Result<String, Error> {
        let header = self.try_get_header()?;

        Ok(header.token().to_string())
    }

    pub fn try_get_header(&self) -> Result<RequestHeader, Error> {
        let header = match self.request.get(HEADER_KEY) {
            Some(header) => match serde_json::from_value::<RequestHeader>(header.clone()) {
                Ok(header) => header,
                Err(error) => {
                    return Err(Error::new(
                        ErrorKind::MalformedRequest,
                        format!("failed to deserialize request header: {}", error),
                    ));
                }
            },
            None => {
                return Err(Error::new(
                    ErrorKind::MalformedRequest,
                    "request has no header",
                ));
            }
        };

        Ok(header)
    }

    pub fn try_get_parameter<T: DeserializeOwned>(&self, key: &str) -> Result<T, Error> {
        match self.request.get(key) {
            Some(raw_value) => match serde_json::from_value::<T>(raw_value.clone()) {
                Ok(value) => Ok(value),
                Err(error) => Err(Error::new(
                    ErrorKind::MalformedRequest,
                    format!("failed to read '{}': {}", key, error),
                )),
            },
            None => Err(Error::new(
                ErrorKind::MalformedRequest,
                format!("request has no '{}'", key),
            )),
        }
    }
}
