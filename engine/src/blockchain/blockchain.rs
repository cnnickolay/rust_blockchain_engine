
use std::collections::HashSet;

use super::{utxo::UnspentOutput, signed_balanced_transaction::{SignedBalancedTransaction}};
use anyhow::{Result, anyhow};
use rsa::RsaPublicKey;

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
        let digest = transaction.hash();

        transaction.signature.verify(&public_key, &digest)?;

        // 4. ensure that all input utxos are unspent
        self.ensure_utxos_unspent(&transaction.inputs)?;

        Ok(())
    }

    pub fn add_transaction(&mut self, transaction: &SignedBalancedTransaction) -> Result<()> {
        self.verify_transaction(&transaction)?;

        self.transactions.push(transaction.clone());

        Ok(())
    }

    /**
     * Makes sure given utxos exist and unspent
     */
    pub fn ensure_utxos_unspent(&self, utxos: &Vec<UnspentOutput>) -> Result<()> {
        let input_utxos: HashSet<String> = HashSet::from_iter(utxos.iter().map(|utxo| utxo.hash_str()));
        let mut remaining_utxos = HashSet::<String>::from_iter(input_utxos.clone());

        // check if at least one utxo has been spent
        for transaction in &self.transactions {
            for utxo in &transaction.inputs {
                let hash = utxo.hash_str();
                if input_utxos.contains(&hash) {
                    return Err(anyhow!("Utxo {} has already been spent", hash));
                }
            }
        }

        // make sure all utxos exist
        remaining_utxos.remove(&self.initial_utxo.hash_str());
        for transaction in &self.transactions {
            for utxo in &transaction.outputs {
                remaining_utxos.remove(&utxo.hash_str());
            }
        }

        if !remaining_utxos.is_empty() {
            return Err(anyhow!("Utxos not found: {}", Vec::from_iter(remaining_utxos.clone()).join(", ")));
        }
        
        Ok(())
    }

}