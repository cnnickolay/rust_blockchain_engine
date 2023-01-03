
use std::collections::HashSet;

use crate::model::{PublicKeyStr, PrivateKeyStr};

use super::{utxo::UnspentOutput, signed_balanced_transaction::{SignedBalancedTransaction}, block::Block};
use anyhow::{Result, anyhow};
use rsa::RsaPublicKey;

pub struct BlockChain {
    pub validator_public_key: PublicKeyStr,
    pub initial_utxo: UnspentOutput,
    pub blocks: Vec<Block>,
}

impl BlockChain {
    pub fn new(validator_public_key: &PublicKeyStr, initial_utxo: UnspentOutput) -> Self {
        Self {
            validator_public_key: validator_public_key.clone(),
            initial_utxo,
            blocks: vec![],
        }
    }

    pub fn new_testing_only(validator_public_key: &PublicKeyStr, initial_utxo: UnspentOutput, blocks: Vec<Block>) -> Self {
        Self {
            validator_public_key: validator_public_key.clone(),
            initial_utxo,
            blocks,
        }
    }

    pub fn verify_transaction(&self, transaction: &SignedBalancedTransaction) -> Result<()> {
        // 1. make sure input amount matches output amount
        transaction.check_balanced()?;

        // 2. make sure there is only one address in inputs (multiple signatures are not supported yet)
        let from_address = transaction.get_from_address()?;

        // 3. make sure signature provided is correct
        let public_key = RsaPublicKey::try_from(from_address)?;
        let cbor = transaction.balanced_transaction.to_cbor()?;

        transaction.signature.verify(&public_key, &cbor)?;

        // 4. ensure that all input utxos are unspent
        self.ensure_utxos_unspent(&transaction.inputs())?;

        Ok(())
    }

    pub fn commit_transaction(&mut self, transaction: &SignedBalancedTransaction, validator_private_key: &PrivateKeyStr) -> Result<Block> {
        self.verify_transaction(&transaction)?;

        let previous_block_hash = if self.blocks.is_empty() {
            self.initial_utxo.hash()
        } else {
            hex::decode(&self.blocks.last().unwrap().hash)?
        };

        let block = Block::create_block_and_sign(&previous_block_hash, transaction, validator_private_key)?;

        self.blocks.push(block.clone());

        Ok(block)
    }

    /**
     * Makes sure given utxos exist and unspent
     */
    pub fn ensure_utxos_unspent(&self, utxos: &Vec<UnspentOutput>) -> Result<()> {
        let input_utxos: HashSet<String> = HashSet::from_iter(utxos.iter().map(|utxo| utxo.hash_str()));
        let mut remaining_utxos = HashSet::<String>::from_iter(input_utxos.clone());

        // check if at least one utxo has been spent
        for block in &self.blocks {
            for utxo in block.transaction.inputs() {
                let hash = utxo.hash_str();
                if input_utxos.contains(&hash) {
                    return Err(anyhow!("Utxo {} has already been spent", hash));
                }
            }
        }

        // make sure all utxos exist
        remaining_utxos.remove(&self.initial_utxo.hash_str());
        for block in &self.blocks {
            for utxo in block.transaction.outputs() {
                remaining_utxos.remove(&utxo.hash_str());
            }
        }

        if !remaining_utxos.is_empty() {
            return Err(anyhow!("Utxos not found: {}", Vec::from_iter(remaining_utxos.clone()).join(", ")));
        }
        
        Ok(())
    }

    pub fn all_balances(&self) -> Vec<(PublicKeyStr, u64)> {
        todo!()
    }

    pub fn blockchain_hash(&self) -> Result<String> {
        let mut last_block_hash = Box::new(self.initial_utxo.hash());
        for (idx, block) in self.blocks.iter().enumerate() {
            if !block.verify_block(&last_block_hash)? {
                return Err(anyhow::anyhow!("Blockchain corrupted at index {}. Verification failed", idx));
            } else {
                let block_hash = hex::decode(&block.hash)?;
                last_block_hash = Box::new(block_hash.clone());
            }
        }
        let hash = hex::encode(last_block_hash.as_slice());
        Ok(hash)
    }

}