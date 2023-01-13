use jsonwebtoken::jwk::JwkSet;

use crate::error::{Error, ErrorKind};

use super::openid_connect_config::OpenIdConnectConfig;

pub struct TokenValidatorConfig {
    jwks: JwkSet,
    openid_connect: OpenIdConnectConfig,
}

impl TokenValidatorConfig {
    pub fn new(jwks: JwkSet, openid_connect: OpenIdConnectConfig) -> TokenValidatorConfig {
        TokenValidatorConfig {
            jwks,
            openid_connect,
        }
    }

    pub fn jwks(&self) -> &JwkSet {
        &self.jwks
    }

    pub fn open_id_connect(&self) -> &OpenIdConnectConfig {
        &self.openid_connect
    }
}

pub async fn try_generate_config(openid_connect: OpenIdConnectConfig) -> Result<TokenValidatorConfig, Error> {
    let jwks = match try_get_jwks(openid_connect.jwks_uri()).await {
        Ok(jwks) => jwks,
        Err(error) => return Err(error),
    };

    Ok(TokenValidatorConfig {
        jwks,
        openid_connect,
    })
}

async fn try_get_jwks(jwks_uri: &str) -> Result<JwkSet, Error> {
    let jwks = match reqwest::get(jwks_uri).await {
        Ok(response) => match response.json::<JwkSet>().await {
            Ok(jwks) => jwks,
            Err(error) => {
                return Err(Error::new(
                    ErrorKind::AutoConfigFailure,
                    format!("failed to deserialize response as JwkSet: {}", error),
                ));
            }
        },
        Err(error) => {
            return Err(Error::new(
                ErrorKind::AutoConfigFailure,
                format!("failed to request jwks: {}", error),
            ));
        }
    };

    Ok(jwks)
}
