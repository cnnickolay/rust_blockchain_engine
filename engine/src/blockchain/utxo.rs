use crate::model::PublicKeyStr;
use serde::{Deserialize, Serialize};
use sha1::Digest;
use sha2::Sha256;

use super::{cbor::Cbor, uuid::Uuid, blockchain::BlockChain};

#[derive(Clone, Eq, Hash, PartialEq, Debug, Serialize, Deserialize)]
pub struct UnspentOutput {
    pub id: UnspentOutputId,
    pub address: PublicKeyStr,
    pub amount: u64,
}

#[derive(Clone, Eq, Hash, PartialEq, Debug, Serialize, Deserialize)]
pub struct UnspentOutputId(pub String);

impl UnspentOutput {
    pub fn to_blockchain(self) -> BlockChain {
        BlockChain::new(self)
    }

    pub fn initial_utxo(address: &PublicKeyStr, amount: u64) -> Self {
        UnspentOutput {
            id: UnspentOutputId("0".to_owned()),
            address: address.clone(),
            amount,
        }
    }

    pub fn new(address: &PublicKeyStr, amount: u64) -> UnspentOutput {
        UnspentOutput {
            id: UnspentOutputId(Uuid::generate().0.to_owned()),
            address: address.clone(),
            amount,
        }
    }

    pub fn hash(&self) -> Vec<u8> {
        let mut hasher = Sha256::new();
        hasher.update(&self.id.0.as_bytes());
        hasher.update(&self.address.0 .0.as_bytes());
        hasher.update(&self.amount.to_le_bytes());
        hasher.finalize().to_vec()
    }

    pub fn hash_str(&self) -> String {
        hex::encode(self.hash())
    }
}

impl TryFrom<&Cbor> for UnspentOutput {
    type Error = anyhow::Error;

    fn try_from(value: &Cbor) -> Result<Self, Self::Error> {
        let cbor_bytes = hex::decode(&value.0)?;
        Ok(serde_cbor::from_slice(&cbor_bytes)?)
    }
}

impl TryFrom<&UnspentOutput> for Cbor {
    type Error = anyhow::Error;

    fn try_from(value: &UnspentOutput) -> Result<Self, Self::Error> {
        let cbor = serde_cbor::to_vec(value)?;
        Ok(Cbor(hex::encode(&cbor)))
    }
}
