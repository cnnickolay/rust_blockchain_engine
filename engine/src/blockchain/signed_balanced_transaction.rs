use crate::model::{Signature, PublicKeyStr, PrivateKeyStr};
use anyhow::{Result, anyhow};
use serde::{Serialize, Deserialize};
use super::{utxo::UnspentOutput, blockchain::BlockChain, transaction_id::TransactionId, balanced_transaction::{BalancedTransaction}, cbor::Cbor, block::Block};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SignedBalancedTransaction {
    pub balanced_transaction: BalancedTransaction,
    pub signature: Signature,
}

impl SignedBalancedTransaction {

    pub fn new(balanced_transaction: &BalancedTransaction, signature: &Signature) -> SignedBalancedTransaction {
        SignedBalancedTransaction {
            balanced_transaction: balanced_transaction.clone(),
            signature: signature.clone(),
        }
    }

    pub fn id(&self) -> &TransactionId {
        &self.balanced_transaction.id
    }

    pub fn inputs(&self) -> &Vec<UnspentOutput> {
        &self.balanced_transaction.inputs
    }

    pub fn outputs(&self) -> &Vec<UnspentOutput> {
        &self.balanced_transaction.outputs
    }

    /**
     * Ensures that input and output balances match.
     */
    pub fn check_balanced(&self) -> Result<()> {
        let mut input_amt: u64 = 0;
        let mut output_amt: u64 = 0;

        for input in self.inputs() {
            input_amt += input.amount;
        }
        for output in self.outputs() {
            output_amt += output.amount;
        }
        if input_amt != output_amt {
            Err(anyhow!("Transaction input and output amounts don't match"))
        } else {
            Ok(())
        }
    }

    /**
     * Returns address from which funds will be sent.
     */
    pub fn get_from_address(&self) -> Result<&PublicKeyStr> {
        if self.inputs().len() == 0 {
            return Err(anyhow!("Transaction has no inputs"));
        } else if self.inputs().len() == 0 {
            Ok(&self.inputs()[0].address)
        } else {
            let mut address: &PublicKeyStr = &self.inputs()[0].address;
            for input in &self.inputs()[1..] {
                if *address != input.address {
                    return Err(anyhow!("Transaction has multiple input addresses, this feature is not supported yet"));
                }
                
                address = &input.address;
            }
            Ok(address)
        }
    }

    pub fn commit(&self, blockchain: &mut BlockChain, validator_private_key: &PrivateKeyStr) -> Result<Block> {
        let block = blockchain.commit_transaction(self, validator_private_key)?;
        Ok(block)
    }

    pub fn hash(&self) -> Result<Vec<u8>> {
        Ok(Cbor::try_from(self)?.hash())
    }
}

impl TryFrom<&Cbor> for SignedBalancedTransaction {
    type Error = anyhow::Error;

    fn try_from(value: &Cbor) -> Result<Self, Self::Error> {
        let cbor_bytes = hex::decode(&value.0)?;
        Ok(serde_cbor::from_slice(&cbor_bytes)?)
    }
}

impl TryFrom<&SignedBalancedTransaction> for Cbor {
    type Error = anyhow::Error;

    fn try_from(value: &SignedBalancedTransaction) -> Result<Self, Self::Error> {
        let cbor = serde_cbor::to_vec(value)?;
        Ok(Cbor(hex::encode(&cbor)))
    }
}