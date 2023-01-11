use std::ops::Index;

use rsa::RsaPrivateKey;

use crate::model::{PrivateKeyStr, PublicKeyStr};

pub type ValidatorPublicKeyAndAddress = (PublicKeyStr, ValidatorAddress);

/**
 * A runtime configuration for current node
 */
pub struct Configuration {
    pub ip: String,
    pub port: u16,
    pub validator_private_key: PrivateKeyStr,
    pub validator_public_key: PublicKeyStr,
    pub validators: Vec<ValidatorPublicKeyAndAddress>,
}

impl Configuration {
    pub fn new(ip: &str, port: u16, validator_private_key: &PrivateKeyStr) -> Self {
        let rsa_public_key = RsaPrivateKey::try_from(validator_private_key).unwrap().to_public_key();
        let public_key = PublicKeyStr::try_from(&rsa_public_key).unwrap();
        Configuration {
            ip: ip.to_string(),
            port,
            validator_private_key: validator_private_key.clone(),
            validator_public_key: public_key,
            validators: Vec::new()
        }
    }

    pub fn address(&self) -> String {
        format!("{}:{}", self.ip, self.port)
    }

    pub fn add_validators(&mut self, new_validators: &[ValidatorPublicKeyAndAddress]) {
        let new_distinct_validators = new_validators.iter().filter(|(validator_pub_key, validator_addr) | {
            *validator_pub_key != self.validator_public_key &&
            self.validators.iter()
                .find(|(existing_validator_pub_key, existing_validator_addr)| 
                    existing_validator_pub_key == validator_pub_key
                ).is_none()
        });
        self.validators.extend(Vec::from_iter(new_distinct_validators.cloned()));
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ValidatorAddress(pub String);

