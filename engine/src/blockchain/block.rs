use rsa::RsaPrivateKey;
use serde::Serialize;
use sha1::Digest;
use sha2::Sha256;
use anyhow::Result;
use crate::model::{Signature, PublicKeyStr, PrivateKeyStr};

use super::{signed_balanced_transaction::SignedBalancedTransaction, cbor::Cbor, validator_signature::ValidatorSignature};

#[derive(Clone, Serialize)]
pub struct Block {
    /**
     * Hash of this block computation is based on hash of the previous block on blockchain plus hash of the transaction
     */
    pub hash: String,
    pub transaction: SignedBalancedTransaction,

    /**
     * Entire block with block hash and transaction hash is signed by validator's private key
     * To resolve contention between validators. 
     */
    pub validator_signatures: Vec<ValidatorSignature>
}

impl Block {
    pub fn create_block_and_sign(previous_block_hash: &[u8], transaction: &SignedBalancedTransaction, validator_private_key: &PrivateKeyStr) -> Result<Block> {
        let mut hasher = Sha256::new();
        hasher.update(previous_block_hash);
        hasher.update(transaction.hash()?);
        let next_block_hash = hex::encode(hasher.finalize().to_vec());

        let private_key = RsaPrivateKey::try_from(validator_private_key)?;
        let transaction_cbor = hex::decode(Cbor::try_from(transaction)?.0)?;
        let validator_signature = Signature::sign(&private_key, &transaction_cbor)?;
        let public_key = PublicKeyStr::try_from(&private_key.to_public_key())?;

        Ok(Block {
            hash: next_block_hash, 
            transaction: transaction.clone(), 
            validator_signatures: vec![ValidatorSignature::new(&public_key, &validator_signature)]
        })
    }

    pub fn verify_block(&self, previous_block_hash: &[u8]) -> Result<bool> {
        let mut hasher = Sha256::new();
        hasher.update(previous_block_hash);
        hasher.update(self.transaction.hash()?);
        let computed_hash = hasher.finalize().to_vec();

        Ok(computed_hash == hex::decode(&self.hash)?)
    }
}