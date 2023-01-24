use protocol::common::ValidatorWithSignature;
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
     * Hash is calculated on the following fields: transaction, original_validator_signature
     */
    pub hash: String,

    /**
     * Balanced transaction, signed by the sending party
     */
    pub transaction: SignedBalancedTransaction,

    /**
     * This field points to the validator which was elected to add this block into the blockchain
     * This validator will get a reward
     */
    original_validator_signature: ValidatorSignature,

    /**
     * Entire block with block hash and transaction hash is signed by validator's private key
     * To resolve contention between validators. 
     */
    validator_signatures: Vec<ValidatorSignature>,
}

impl Block {
    pub fn create_block_and_sign(previous_block_hash: &[u8], transaction: &SignedBalancedTransaction, validator_private_key: &PrivateKeyStr) -> Result<Block> {
        let private_key = RsaPrivateKey::try_from(validator_private_key)?;
        let public_key = PublicKeyStr::try_from(&private_key.to_public_key())?;
        let transaction_cbor = hex::decode(Cbor::try_from(transaction)?.0)?;
        let validator_signature = Signature::sign(&private_key, &transaction_cbor)?;
        let original_validator_signature = ValidatorSignature::new(&public_key, &validator_signature);

        let mut hasher = Sha256::new();
        hasher.update(previous_block_hash);
        hasher.update(transaction.hash()?);
        hasher.update(Cbor::try_from(&original_validator_signature)?.0);
        let next_block_hash = hex::encode(hasher.finalize().to_vec());

        Ok(Block {
            hash: next_block_hash, 
            transaction: transaction.clone(), 
            validator_signatures: vec![original_validator_signature.clone()],
            original_validator_signature,
        })
    }

    pub fn verify_block(&self, previous_block_hash: &[u8]) -> Result<bool> {
        let mut hasher = Sha256::new();
        hasher.update(previous_block_hash);
        hasher.update(self.transaction.hash()?);
        let computed_hash = hasher.finalize().to_vec();

        Ok(computed_hash == hex::decode(&self.hash)?)
    }

    pub fn add_validator_signature(&mut self, signature: ValidatorSignature) {
        if let Some(_) = self.validator_signatures.iter().find(|_signature| **_signature == signature) {
            return;
        }
        self.validator_signatures.push(signature);
    }

    pub fn validator_signatures(&self) -> &[ValidatorSignature] {
        &self.validator_signatures
    }
}

impl From<&ValidatorWithSignature> for ValidatorSignature {
    fn from(v: &ValidatorWithSignature) -> Self {
        ValidatorSignature::new(
            &PublicKeyStr::from_str(&v.validator.public_key),
            &Signature::from_string(&v.signature)
        )
    }
}