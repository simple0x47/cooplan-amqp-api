use serde_json::{Map, Value};
use crate::api::input::request::Request;

use crate::error::{Error, ErrorKind};

pub fn sanitize(
    raw_request: Map<String, Value>,
    actions: &'static [&'static str],
) -> Result<Request, Error> {
    let request = Request::new(raw_request);

    let header = request.try_get_header()?;

    if !actions
        .iter()
        .any(|valid_action| (*valid_action).eq(header.action()))
    {
        return Err(Error::new(
            ErrorKind::SanitizationFailure,
            format!("invalid action detected: {}", header.action()),
        ));
    }

    Ok(request)
}
