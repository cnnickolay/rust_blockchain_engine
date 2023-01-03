use std::collections::HashSet;
use anyhow::Result;
use rsa::RsaPrivateKey;

use crate::model::{PrivateKeyStr, PublicKeyStr};

/**
 * A runtime configuration for current node
 */
pub struct Configuration {
    pub ip: String,
    pub port: u16,
    pub validator_private_key: PrivateKeyStr,
    pub validators: HashSet<(PublicKeyStr, ValidatorAddress)>,
}

impl Configuration {
    pub fn new(ip: &str, port: u16, validator_private_key: &PrivateKeyStr) -> Self {
        Configuration {
            ip: ip.to_string(),
            port,
            validator_private_key: validator_private_key.clone(),
            validators: HashSet::new()
        }
    }

    pub fn public_key(&self) -> Result<PublicKeyStr> {
        let private_key = RsaPrivateKey::try_from(&self.validator_private_key)?;
        let public_key = private_key.to_public_key();
        Ok(PublicKeyStr::try_from(&public_key)?)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ValidatorAddress(pub String);
