use cooplan_amqp_api_shared::api::input::request_result_error::{RequestResultError, RequestResultErrorKind};

use crate::error::{Error, ErrorKind};

impl From<ErrorKind> for RequestResultErrorKind {
    fn from(kind: ErrorKind) -> Self {
        match kind {
            ErrorKind::MalformedRequest => RequestResultErrorKind::MalformedRequest,
            _ => RequestResultErrorKind::InternalFailure,
        }
    }
}

impl From<Error> for RequestResultError {
    fn from(error: Error) -> Self {
        let kind: RequestResultErrorKind = error.kind.into();

        RequestResultError::new(kind, error.message)
    }
}
