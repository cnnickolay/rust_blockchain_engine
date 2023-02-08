use crate::model::{PrivateKeyStr, PublicKeyStr, Signature};
use anyhow::Result;
use protocol::common::ValidatorWithSignature;
use rsa::RsaPrivateKey;
use serde::Serialize;
use sha1::Digest;
use sha2::Sha256;

use super::{
    cbor::Cbor, signed_balanced_transaction::SignedBalancedTransaction,
    validator_signature::ValidatorSignature,
};

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
     * This field points to the validator which was elected and his signature for the transaction
     * This validator will append the blockchain with its block and will get a reward
     */
    elected_validator_signature: ValidatorSignature,

    /**
     * Validator signatures for combination of previous block hash and public key of elected validator
     */
    votes: Vec<ValidatorSignature>,
}

impl Block {
    pub fn create_block_and_sign(
        previous_block_hash: &[u8],
        transaction: &SignedBalancedTransaction,
        validator_private_key: &PrivateKeyStr,
    ) -> Result<Block> {
        let private_key = RsaPrivateKey::try_from(validator_private_key)?;
        let public_key = PublicKeyStr::try_from(&private_key.to_public_key())?;
        let transaction_cbor = hex::decode(Cbor::try_from(transaction)?.0)?;
        let validator_signature = Signature::sign(&private_key, &transaction_cbor)?;
        let original_validator_signature =
            ValidatorSignature::new(&public_key, &validator_signature);

        let mut hasher = Sha256::new();
        hasher.update(previous_block_hash);
        hasher.update(transaction.hash()?);
        hasher.update(Cbor::try_from(&original_validator_signature)?.0);
        let next_block_hash = hex::encode(hasher.finalize().to_vec());

        Ok(Block {
            hash: next_block_hash,
            transaction: transaction.clone(),
            elected_validator_signature: original_validator_signature,
            votes: Vec::new(),
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
        if let Some(_) = self
            .votes
            .iter()
            .find(|_signature| **_signature == signature)
        {
            return;
        }
        self.votes.push(signature);
    }

    pub fn validator_signatures(&self) -> &[ValidatorSignature] {
        &self.votes
    }
}

impl From<&ValidatorWithSignature> for ValidatorSignature {
    fn from(v: &ValidatorWithSignature) -> Self {
        ValidatorSignature::new(
            &PublicKeyStr::from_str(&v.validator.public_key),
            &Signature::from_string(&v.signature),
        )
    }
}
