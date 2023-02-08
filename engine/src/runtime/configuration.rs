use anyhow::Result;
use protocol::common::Validator;
use rsa::RsaPrivateKey;
use serde::{Serialize, Deserialize};

use crate::{
    blockchain::blockchain::BlockChain,
    encryption::generate_rsa_keypair_custom,
    model::{PrivateKeyStr, PublicKeyStr},
};

use super::validator_runtime::ValidatorRuntime;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct ValidatorReference {
    pub pk: PublicKeyStr,
    pub address: ValidatorAddress,
}

impl ValidatorReference {
    pub fn new(pk: &PublicKeyStr, address: &str) -> ValidatorReference {
        ValidatorReference { pk: pk.clone(), address: ValidatorAddress(address.to_owned()) }
    }
}

/**
 * A runtime configuration for current node
 */
#[derive(Clone)]
pub struct Configuration {
    pub ip: String,
    pub port: u16,
    pub validator_private_key: PrivateKeyStr,
    pub validator_public_key: PublicKeyStr,
}

impl Configuration {
    pub fn new(ip: &str, port: u16, validator_private_key: &PrivateKeyStr) -> Self {
        let rsa_public_key = RsaPrivateKey::try_from(validator_private_key)
            .unwrap()
            .to_public_key();
        let public_key = PublicKeyStr::try_from(&rsa_public_key).unwrap();
        Configuration {
            ip: ip.to_string(),
            port,
            validator_private_key: validator_private_key.clone(),
            validator_public_key: public_key,
        }
    }

    pub fn generate_new(ip: &str, port: u16) -> Result<Self> {
        let (sk, _) = generate_rsa_keypair_custom()?;
        let configuration = Configuration::new(ip, port, &sk);
        Ok(configuration)
    }

    pub fn address(&self) -> String {
        format!("{}:{}", self.ip, self.port)
    }

    pub fn validator_ref(&self) -> ValidatorReference {
        ValidatorReference {
            pk: self.validator_public_key.clone(),
            address: ValidatorAddress(format!("{}:{}", self.ip, self.port)),
        }
    }

    pub fn validator(&self) -> Validator {
        Validator::from(&self.validator_ref())
    }

    pub fn to_runtime(self, blockchain: BlockChain, remote_validators: Vec<ValidatorReference>) -> ValidatorRuntime {
        ValidatorRuntime::new(self, blockchain, remote_validators)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash, Default)]
pub struct ValidatorAddress(pub String);

impl TryFrom<&Validator> for ValidatorReference {
    type Error = anyhow::Error;
    fn try_from(v: &Validator) -> Result<Self> {
        v.address.clone().map(|address| {
            ValidatorReference {
                pk: PublicKeyStr::from_str(&v.public_key),
                address: ValidatorAddress(address)
            }
        }).ok_or(anyhow::anyhow!("Unable to convert Validator to ValidatorReference, because there was no ip address included"))
    }
}

impl From<&ValidatorReference> for Validator {
    fn from(v: &ValidatorReference) -> Self {
        Validator {
            address: Some(v.address.0.to_owned()),
            public_key: v.pk.0 .0.to_owned(),
        }
    }
}
