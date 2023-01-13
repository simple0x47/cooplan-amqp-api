use serde::Deserialize;

#[derive(Deserialize)]
pub struct RequestHeader {
    token: String,
    element: String,
    action: String,
}

impl RequestHeader {
    pub fn token(&self) -> &str {
        self.token.as_str()
    }

    pub fn element(&self) -> &str {
        self.element.as_str()
    }

    pub fn action(&self) -> &str {
        self.action.as_str()
    }
}
