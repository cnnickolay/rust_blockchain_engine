
use std::collections::HashSet;

use crate::model::PublicKeyStr;

use super::{utxo::UnspentOutput, signed_balanced_transaction::{SignedBalancedTransaction}, cbor::Cbor};
use anyhow::{Result, anyhow};
use rsa::RsaPublicKey;
use sha1::Digest;
use sha2::Sha256;

pub struct BlockChain {
    pub initial_utxo: UnspentOutput,
    pub transactions: Vec<SignedBalancedTransaction>,
}

impl BlockChain {
    pub fn new(initial_utxo: UnspentOutput) -> Self {
        Self {
            initial_utxo,
            transactions: vec![],
        }
    }

    pub fn new_testing(initial_utxo: UnspentOutput, transactions: Vec<SignedBalancedTransaction>) -> Self {
        Self {
            initial_utxo,
            transactions,
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

    pub fn commit_transaction(&mut self, transaction: &SignedBalancedTransaction) -> Result<String> {
        self.verify_transaction(&transaction)?;

        self.transactions.push(transaction.clone());

        let blockchain_hash = self.blockchain_hash()?;
        Ok(blockchain_hash)
    }

    /**
     * Makes sure given utxos exist and unspent
     */
    pub fn ensure_utxos_unspent(&self, utxos: &Vec<UnspentOutput>) -> Result<()> {
        let input_utxos: HashSet<String> = HashSet::from_iter(utxos.iter().map(|utxo| utxo.hash_str()));
        let mut remaining_utxos = HashSet::<String>::from_iter(input_utxos.clone());

        // check if at least one utxo has been spent
        for transaction in &self.transactions {
            for utxo in transaction.inputs() {
                let hash = utxo.hash_str();
                if input_utxos.contains(&hash) {
                    return Err(anyhow!("Utxo {} has already been spent", hash));
                }
            }
        }

        // make sure all utxos exist
        remaining_utxos.remove(&self.initial_utxo.hash_str());
        for transaction in &self.transactions {
            for utxo in transaction.outputs() {
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
        let mut hasher = Sha256::new();
        hasher.update(Cbor::try_from(&self.initial_utxo)?.hash());

        for transaction in &self.transactions {
            let hash = Cbor::try_from(transaction)?.hash();
            hasher.update(hash);
        }
        let hash = hex::encode(hasher.finalize().to_vec());
        Ok(hash)
    }

}