use std::fmt;

use serde::{Deserialize, Serialize};

use crate::error::{Error, ErrorKind};

#[derive(Copy, Clone, Deserialize, Serialize)]
pub enum RequestResultErrorKind {
    InternalFailure,
    MalformedRequest,
}

impl From<ErrorKind> for RequestResultErrorKind {
    fn from(kind: ErrorKind) -> Self {
        match kind {
            ErrorKind::MalformedRequest => RequestResultErrorKind::MalformedRequest,
            _ => RequestResultErrorKind::InternalFailure,
        }
    }
}

#[derive(Deserialize, Serialize)]
pub struct RequestResultError {
    kind: RequestResultErrorKind,
    message: String,
}

impl RequestResultError {
    pub fn new(kind: RequestResultErrorKind, message: impl Into<String>) -> RequestResultError {
        RequestResultError {
            kind,
            message: message.into(),
        }
    }

    pub fn kind(&self) -> RequestResultErrorKind {
        self.kind
    }
}

impl From<Error> for RequestResultError {
    fn from(error: Error) -> Self {
        let kind: RequestResultErrorKind = error.kind.into();

        RequestResultError::new(kind, error.message)
    }
}

impl fmt::Display for RequestResultError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}
