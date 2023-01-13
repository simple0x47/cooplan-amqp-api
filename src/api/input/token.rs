use std::collections::HashMap;

use jsonwebtoken::TokenData;
use serde_json::Value;

use crate::error::{Error, ErrorKind};

#[derive(Debug)]
pub struct Token {
    token_data: TokenData<HashMap<String, Value>>,
}

const PERMISSIONS_CLAIM: &str = "permissions";

impl Token {
    pub fn new(token_data: TokenData<HashMap<String, Value>>) -> Token {
        Token { token_data }
    }

    pub fn has_permission(&self, permission: &String) -> Result<(), Error> {
        let permissions = match self.token_data.claims.get(PERMISSIONS_CLAIM) {
            Some(permissions) => match serde_json::from_value::<Vec<String>>(permissions.clone()) {
                Ok(permissions) => permissions,
                Err(error) => {
                    return Err(Error::new(
                        ErrorKind::MalformedToken,
                        format!(
                            "failed to deserialize permissions as a strings vector: {}",
                            error
                        ),
                    ))
                }
            },
            None => {
                return Err(Error::new(
                    ErrorKind::MalformedToken,
                    "'permissions' claim is missing",
                ))
            }
        };

        if !permissions.contains(permission) {
            return Err(Error::new(
                ErrorKind::PermissionNotFound,
                format!("permission '{}' could not be found", permission),
            ));
        }

        Ok(())
    }

    pub fn get(&self, key: &str) -> Option<&Value> {
        self.token_data.claims.get(key)
    }
}
