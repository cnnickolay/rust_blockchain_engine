use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Validator {
    pub address: Option<String>,
    pub public_key: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ValidatorWithSignature {
    pub validator: Validator,
    pub signature: String,
}
