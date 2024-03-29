use std::collections::HashMap;

use jsonwebtoken::TokenData;
use serde_json::Value;

use crate::error::{Error, ErrorKind};

const PERMISSIONS_CLAIM: &str = "permissions";
/// Custom claim used in order to support easy assignations of permissions to users
/// without having to figure out a way to edit Auth0's permissions claim.
/// USED ONLY WHENEVER THE AUTH0'S PERMISSIONS CLAIM IS EMPTY.
const CUSTOM_PERMISSIONS_CLAIM: &str = "permission";

pub const USER_ID_CLAIM: &str = "sub";
pub const ORGANIZATION_ID_CLAIM: &str = "organization_id";

#[derive(Debug)]
pub struct Token {
    token_data: TokenData<HashMap<String, Value>>,
    permissions: Vec<String>,
}

impl Token {
    pub fn try_new(token_data: TokenData<HashMap<String, Value>>) -> Result<Token, Error> {
        let mut permissions = get_permissions_from_claim(&token_data, PERMISSIONS_CLAIM)?;

        // Use custom permissions claim *ONLY* if Auth0's permission claim is empty.
        if permissions.is_empty() {
            permissions = get_permissions_from_claim(&token_data, CUSTOM_PERMISSIONS_CLAIM)?;
        }

        Ok(Token {
            token_data,
            permissions,
        })
    }

    pub fn has_permission(&self, permission: &String) -> Result<(), Error> {
        if !self.permissions.contains(permission) {
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

fn get_permissions_from_claim(
    token_data: &TokenData<HashMap<String, Value>>,
    claim: &str,
) -> Result<Vec<String>, Error> {
    let permissions = match token_data.claims.get(claim) {
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
