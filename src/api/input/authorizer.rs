use crate::api::input::request::Request;
use crate::api::input::request_header::RequestHeader;
use crate::api::input::token_validator;
use crate::api::input::token_validator::TokenValidator;
use crate::config::openid_connect_config::OpenIdConnectConfig;
use crate::error::Error;

pub struct Authorizer {
    token_validator: TokenValidator,
}

impl Authorizer {
    pub fn new(token_validator: TokenValidator) -> Authorizer {
        Authorizer { token_validator }
    }

    pub fn authorize(&self, mut request: Request) -> Result<Request, Error> {
        let raw_token = request.try_get_token()?;

        let token = self.token_validator.validate(raw_token.as_str())?;

        let header = request.try_get_header()?;

        let required_permission = permission_from_header(header);

        token.has_permission(&required_permission)?;

        request.authorized_token = Some(token);

        Ok(request)
    }
}

fn permission_from_header(header: RequestHeader) -> String {
    format!("{}:{}", header.action(), header.element())
}

pub async fn try_generate_authorizer(
    openid_connect: OpenIdConnectConfig,
) -> Result<Authorizer, Error> {
    let token_validator = token_validator::try_generate_token_validator(openid_connect).await?;

    Ok(Authorizer::new(token_validator))
}
