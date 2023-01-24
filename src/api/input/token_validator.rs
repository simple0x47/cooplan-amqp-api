use std::collections::HashMap;

use crate::api::input::token::Token;
use crate::config::openid_connect_config::OpenIdConnectConfig;
use jsonwebtoken::{decode, decode_header, jwk::AlgorithmParameters, DecodingKey, Validation};
use serde_json::Value;

use crate::config::token_validator_config;
use crate::config::token_validator_config::TokenValidatorConfig;
use crate::error::{Error, ErrorKind};

pub struct TokenValidator {
    config: TokenValidatorConfig,
}

impl TokenValidator {
    pub fn new(config: TokenValidatorConfig) -> TokenValidator {
        TokenValidator { config }
    }

    pub fn validate(&self, token: &str) -> Result<Token, Error> {
        let header = match decode_header(token) {
            Ok(header) => header,
            Err(error) => {
                return Err(Error::new(
                    ErrorKind::TokenDecodingFailure,
                    format!("failure to decode token's header: {}", error),
                ));
            }
        };

        let kid = match header.kid {
            Some(k) => k,
            None => {
                return Err(Error::new(
                    ErrorKind::MalformedToken,
                    "failed to find token header's kid",
                ));
            }
        };

        let jwk = match self.config.jwks().find(&kid) {
            Some(jwk) => jwk,
            None => {
                return Err(Error::new(
                    ErrorKind::MalformedToken,
                    format!("failed to find jwk for kid '{}'", kid),
                ));
            }
        };

        let rsa = match jwk.algorithm {
            AlgorithmParameters::RSA(ref rsa) => rsa,
            _ => {
                return Err(Error::new(
                    ErrorKind::MalformedToken,
                    format!("expected 'RSA' algorithm got '{:?}'", jwk.algorithm),
                ));
            }
        };

        let decoding_key = match DecodingKey::from_rsa_components(&rsa.n, &rsa.e) {
            Ok(decoding_key) => decoding_key,
            Err(error) => {
                return Err(Error::new(
                    ErrorKind::TokenDecodingFailure,
                    format!("failed to get decoding key: {}", error),
                ));
            }
        };

        let algorithm = match jwk.common.algorithm {
            Some(algorithm) => algorithm,
            None => {
                return Err(Error::new(
                    ErrorKind::TokenDecodingFailure,
                    "jwk is missing algorithm",
                ));
            }
        };

        let mut validation = Validation::new(algorithm);
        validation.validate_exp = true;
        validation.set_audience(self.config.open_id_connect().audience());
        validation.set_issuer(self.config.open_id_connect().issuers());

        let decoded_token =
            match decode::<HashMap<String, Value>>(token, &decoding_key, &validation) {
                Ok(decoded_token) => decoded_token,
                Err(error) => {
                    return Err(Error::new(
                        ErrorKind::InvalidToken,
                        format!("invalid token detected: {}", error),
                    ));
                }
            };

        let wrapped_token = Token::try_new(decoded_token)?;
        Ok(wrapped_token)
    }
}

pub async fn try_generate_token_validator(
    openid_connect: OpenIdConnectConfig,
) -> Result<TokenValidator, Error> {
    let token_validator_config =
        token_validator_config::try_generate_config(openid_connect).await?;

    Ok(TokenValidator::new(token_validator_config))
}
