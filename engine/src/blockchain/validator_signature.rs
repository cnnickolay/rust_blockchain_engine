use serde::Serialize;

use crate::model::{PublicKeyStr, Signature};

#[derive(Clone, Serialize)]
pub struct ValidatorSignature {
    pub validator_public_key: PublicKeyStr,
    pub validator_signature: Signature
}

impl ValidatorSignature {
    pub fn new(validator_public_key: &PublicKeyStr, validator_signature: &Signature) -> ValidatorSignature {
        ValidatorSignature { validator_public_key: validator_public_key.clone(), validator_signature: validator_signature.clone() }
    }
}