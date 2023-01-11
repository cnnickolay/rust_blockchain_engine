use std::collections::HashSet;

use crate::model::PublicKeyStr;
use anyhow::{Result, anyhow};

use super::{blockchain::BlockChain, utxo::UnspentOutput, balanced_transaction::BalancedTransaction, transaction_id::TransactionId};

pub struct Transaction {
    pub from: PublicKeyStr,
    pub to: PublicKeyStr,
    pub amount: u64,
}

impl Transaction {
    pub fn new(from: &PublicKeyStr, to: &PublicKeyStr, amount: u64) -> Transaction {
        Transaction { from: from.clone(), to: to.clone(), amount }
    }

    pub fn balance_transaction(&self, blockchain: &BlockChain) -> Result<BalancedTransaction> {
        let mut unspent_utxos = HashSet::new();
        if self.from == blockchain.initial_utxo.address {
            unspent_utxos.insert(blockchain.initial_utxo.clone());
        }

        for block in &blockchain.blocks {
            for utxo in block.transaction.outputs() {
                if utxo.address == self.from {
                    unspent_utxos.insert(utxo.clone());
                }
            }
        }

        for block in &blockchain.blocks {
            for utxo in block.transaction.inputs() {
                if utxo.address == self.from {
                    unspent_utxos.remove(utxo);
                }
            }
        }

        let mut amt = 0 as u64;
        let mut selected_utxos: Vec<UnspentOutput> = Vec::new();
        for utxo in unspent_utxos.iter() {
            if amt >= self.amount {
                break;
            } else {
                amt += utxo.amount;
                selected_utxos.push(utxo.clone());
            }
        }

        if amt < self.amount {
            return Err(anyhow!("Not enough funds for {}", self.from));
        }

        let mut output_utxos: Vec<UnspentOutput> = Vec::new();
        if amt > self.amount {
            let change_utxo = UnspentOutput::new(&self.from.clone(), amt - self.amount);
            output_utxos.push(change_utxo);
        }
        let transfer_utxo = UnspentOutput::new(&self.to.clone(), self.amount);
        output_utxos.push(transfer_utxo);

        Ok(BalancedTransaction {
            id: TransactionId::generate(),
            inputs: selected_utxos,
            outputs: output_utxos,
        })
    }

}