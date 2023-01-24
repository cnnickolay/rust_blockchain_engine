use protocol::common::{ValidatorWithSignature, Validator};
use serde::{Serialize, Deserialize};

use crate::model::{PublicKeyStr, Signature};

#[derive(Clone, Serialize, PartialEq, Eq, Hash, Deserialize)]
pub struct ValidatorSignature {
    pub validator_public_key: PublicKeyStr,
    pub validator_signature: Signature
}

impl ValidatorSignature {
    pub fn new(validator_public_key: &PublicKeyStr, validator_signature: &Signature) -> ValidatorSignature {
        ValidatorSignature { validator_public_key: validator_public_key.clone(), validator_signature: validator_signature.clone() }
    }
}

impl From<&ValidatorSignature> for ValidatorWithSignature {
    fn from(v: &ValidatorSignature) -> Self {
        ValidatorWithSignature { 
            validator: Validator { 
                address: None, 
                public_key: v.validator_public_key.0.0.to_owned(), 
            }, 
            signature: v.validator_signature.0.0.to_owned()
        }
    }
}
