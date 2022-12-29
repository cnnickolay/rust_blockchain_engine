use sha1::Digest;
use sha2::Sha256;
use crate::model::PublicKeyStr;

use super::uuid::Uuid;

#[derive(Clone, Eq, Hash, PartialEq, Debug)]
pub struct UnspentOutput {
    pub id: UnspentOutputId,
    pub address: PublicKeyStr,
    pub amount: u64,
}

#[derive(Clone, Eq, Hash, PartialEq, Debug)]
pub struct UnspentOutputId(pub Uuid);

impl UnspentOutput {
    // pub fn new(uuid: String, address: PublicKeyStr, amount: u64) -> UnspentOutput {
    //     UnspentOutput {
    //         id: UnspentOutputId(Uuid::new(&uuid)),
    //         address,
    //         amount,
    //     }
    // }

    pub fn new(address: &PublicKeyStr, amount: u64) -> UnspentOutput {
        UnspentOutput {
            id: UnspentOutputId(Uuid::generate()),
            address: address.clone(),
            amount,
        }
    }

    pub fn hash(&self) -> Vec<u8> {
        let mut hasher = Sha256::new();
        hasher.update(&self.id.0.0.as_bytes());
        hasher.update(&self.address.0.0.as_bytes());
        hasher.update(&self.amount.to_le_bytes());
        hasher.finalize().to_vec()
    }

    pub fn hash_str(&self) -> String {
        hex::encode(self.hash())
    }
}