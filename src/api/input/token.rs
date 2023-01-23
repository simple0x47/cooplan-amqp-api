use std::collections::HashMap;

use jsonwebtoken::TokenData;
use serde_json::Value;

use crate::error::{Error, ErrorKind};

#[derive(Debug)]
pub struct Token {
    token_data: TokenData<HashMap<String, Value>>,
}

const PERMISSIONS_CLAIM: &str = "permissions";
/// Custom claim used in order to support easy assignations of permissions to users
/// without having to figure out a way to edit Auth0's permissions claim.
/// USED ONLY WHENEVER THE AUTH0'S PERMISSIONS CLAIM IS EMPTY.
const CUSTOM_PERMISSIONS_CLAIM: &str = "permission";

impl Token {
    pub fn new(token_data: TokenData<HashMap<String, Value>>) -> Token {
        Token { token_data }
    }

    pub fn has_permission(&self, permission: &String) -> Result<(), Error> {
        let mut permissions = self.get_permissions_from_claim(PERMISSIONS_CLAIM)?;

        // Use custom permissions claim *ONLY* if Auth0's permission claim is empty.
        if permissions.is_empty() {
            permissions = self.get_permissions_from_claim(CUSTOM_PERMISSIONS_CLAIM)?;
        }

        if !permissions.contains(permission) {
            return Err(Error::new(
                ErrorKind::PermissionNotFound,
                format!("permission '{}' could not be found", permission),
            ));
        }

        Ok(())
    }

    fn get_permissions_from_claim(&self, claim: &str) -> Result<Vec<String>, Error> {
        let permissions = match self.token_data.claims.get(claim) {
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

        Ok(permissions)
    }

    pub fn get(&self, key: &str) -> Option<&Value> {
        self.token_data.claims.get(key)
    }
}
