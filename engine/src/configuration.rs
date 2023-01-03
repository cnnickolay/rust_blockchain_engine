use crate::model::PrivateKeyStr;

/**
 * A runtime configuration for current node
 */
pub struct Configuration {
    pub ip: String,
    pub port: u16,
    pub validator_private_key: PrivateKeyStr,
    pub validators: Vec<ValidatorAddress>,
}

impl Configuration {
    pub fn new(ip: &str, port: u16, validator_private_key: &PrivateKeyStr) -> Self {
        Configuration {
            ip: ip.to_string(),
            port,
            validator_private_key: validator_private_key.clone(),
            validators: Vec::new()
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ValidatorAddress(pub String);
