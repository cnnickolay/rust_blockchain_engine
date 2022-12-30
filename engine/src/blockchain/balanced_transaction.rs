use rsa::RsaPrivateKey;
use serde::{Serialize, Deserialize};
use anyhow::Result;
use crate::model::Signature;

use super::{utxo::UnspentOutput, transaction_id::TransactionId, signed_balanced_transaction::SignedBalancedTransaction, cbor::Cbor};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BalancedTransaction {
    pub id: TransactionId,
    pub inputs: Vec<UnspentOutput>,
    pub outputs: Vec<UnspentOutput>,
}

impl BalancedTransaction {
    pub fn to_cbor(&self) -> Result<Vec<u8>> {
        Ok(serde_cbor::to_vec(self)?)
    }

    pub fn sign(&self, private_key: &RsaPrivateKey) -> Result<SignedBalancedTransaction> {
        let cbor = self.to_cbor()?;

        let signature = Signature::sign(&private_key, &cbor)?;

        Ok(SignedBalancedTransaction {
            balanced_transaction: self.clone(),
            signature,
        })
    }
}

impl TryFrom<&Cbor> for BalancedTransaction {
    type Error = anyhow::Error;

    fn try_from(value: &Cbor) -> Result<Self, Self::Error> {
        let cbor_bytes = hex::decode(&value.0)?;
        Ok(serde_cbor::from_slice(&cbor_bytes)?)
    }
}

impl TryFrom<&BalancedTransaction> for Cbor {
    type Error = anyhow::Error;

    fn try_from(value: &BalancedTransaction) -> Result<Self, Self::Error> {
        let cbor = serde_cbor::to_vec(value)?;
        Ok(Cbor(hex::encode(&cbor)))
    }
}