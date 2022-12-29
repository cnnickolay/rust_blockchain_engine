use crate::model::{Signature, PublicKeyStr};
use anyhow::{Result, anyhow};
use rsa::RsaPrivateKey;
use sha1::Digest;
use sha2::Sha256;
use super::{utxo::UnspentOutput, uuid::Uuid, blockchain::BlockChain};


#[derive(Debug)]
pub struct BalancedTransaction {
    pub id: TransactionId,
    pub inputs: Vec<UnspentOutput>,
    pub outputs: Vec<UnspentOutput>,
}

impl BalancedTransaction {
    pub fn hash(&self) -> Vec<u8> {
        let mut hasher = Sha256::new();
        hasher.update(&self.id.0.0.as_bytes());
        for input in &self.inputs {
            hasher.update(input.hash());
        }
        for output in &self.outputs {
            hasher.update(output.hash());
        }
        hasher.finalize().to_vec()
    }

    pub fn sign(&self, private_key: RsaPrivateKey) -> Result<SignedBalancedTransaction> {
        let hash = self.hash();

        let signature = Signature::sign(&private_key, &hash)?;

        Ok(SignedBalancedTransaction {
            id: self.id.clone(),
            inputs: self.inputs.clone(),
            outputs: self.outputs.clone(),
            signature,
        })
    }
}

#[derive(Clone, Debug)]
pub struct SignedBalancedTransaction {
    pub id: TransactionId,
    pub inputs: Vec<UnspentOutput>,
    pub outputs: Vec<UnspentOutput>,
    pub signature: Signature,
}

impl SignedBalancedTransaction {

    pub fn new(id: &TransactionId, inputs: &Vec<UnspentOutput>, outputs: &Vec<UnspentOutput>, signature: &Signature) -> SignedBalancedTransaction {
        SignedBalancedTransaction {
            id: id.clone(),
            inputs: inputs.clone(),
            outputs: outputs.clone(),
            signature: signature.clone(),
        }
    }

    /**
     * Ensures that input and output balances match.
     */
    pub fn check_balanced(&self) -> Result<()> {
        let mut input_amt: u64 = 0;
        let mut output_amt: u64 = 0;

        for input in &self.inputs {
            input_amt += input.amount;
        }
        for output in &self.outputs {
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
        if self.inputs.len() == 0 {
            return Err(anyhow!("Transaction has no inputs"));
        } else if self.inputs.len() == 0 {
            Ok(&self.inputs[0].address)
        } else {
            let mut address: &PublicKeyStr = &self.inputs[0].address;
            for input in &self.inputs[1..] {
                if *address != input.address {
                    return Err(anyhow!("Transaction has multiple input addresses, this feature is not supported yet"));
                }
                
                address = &input.address;
            }
            Ok(address)
        }
    }

    pub fn hash(&self) -> Vec<u8> {
        let mut hasher = Sha256::new();
        hasher.update(&self.id.0.0.as_bytes());
        for input in &self.inputs {
            hasher.update(input.hash());
        }
        for output in &self.outputs {
            hasher.update(output.hash());
        }
        hasher.finalize().to_vec()
    }

    pub fn verify_and_commit(&self, blockchain: &mut BlockChain) -> Result<SignedBalancedTransaction> {
        blockchain.add_transaction(self)?;
        Ok(self.clone())
    }
}

#[derive(Clone, Debug)]
pub struct TransactionId(pub Uuid);

impl TransactionId {
    pub fn new(id: &str) -> TransactionId {
        TransactionId(Uuid::new(id))
    }

    pub fn generate() -> TransactionId {
        TransactionId(Uuid::generate())
    }

}