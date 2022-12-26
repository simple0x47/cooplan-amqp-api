use serde::{Deserialize, Serialize};
use serde_json::Value;
use crate::api::input::request_result_error::RequestResultError;

use crate::error::Error;

#[derive(Deserialize, Serialize)]
pub enum RequestResult {
    Ok(Value),
    Err(RequestResultError),
}

impl From<Result<(), Error>> for RequestResult {
    fn from(result: Result<(), Error>) -> RequestResult {
        match result {
            Ok(_) => RequestResult::Ok(Value::Null),
            Err(error) => RequestResult::Err(error.into()),
        }
    }
}

impl From<Result<Value, Error>> for RequestResult {
    fn from(result: Result<Value, Error>) -> RequestResult {
        match result {
            Ok(value) => RequestResult::Ok(value),
            Err(error) => RequestResult::Err(error.into()),
        }
    }
}
