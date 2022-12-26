use std::collections::HashMap;
use anyhow::Result;
use sha1::Digest;
use sha2::Sha256;
use uuid::Uuid;
use super::{block::Block, public_key_str::PublicKeyStr, transaction::Transaction};



pub struct BlockChain {
    pub initial_block: InitialBlock,
    pub blocks: Vec<Block>,
    pub nonces: HashMap<PublicKeyStr, String>,
}

pub struct InitialBlock {
    pub address: PublicKeyStr,
    pub balance: u64,
}

impl BlockChain {
    pub fn new(address: &PublicKeyStr, balance: u64) -> Self {
        BlockChain {
            initial_block: InitialBlock {
                address: address.clone(),
                balance,
            },
            blocks: Vec::new(),
            nonces: HashMap::new(),
        }
    }

    pub fn from_vector(
        initial_address: &PublicKeyStr,
        initial_balance: u64,
        signed_transactions: &[Transaction],
    ) -> Result<Self> {
        let mut blockchain = BlockChain::new(initial_address, initial_balance);

        for transaction in signed_transactions {
            blockchain.append_blockchain((*transaction).clone())?;
        }

        Ok(blockchain)
    }

    pub fn balance_for_address(&self, address: &PublicKeyStr) -> Result<u64> {
        let mut balance = if self.initial_block.address == *address {
            self.initial_block.balance
        } else {
            0
        };

        for block in &self.blocks {
            if block.transaction.from_address == *address {
                balance -= block.transaction.amount;
            } else if block.transaction.to_address == *address {
                balance += block.transaction.amount;
            }
        }
        Ok(balance)
    }

    pub fn request_nonce_for_address(&mut self, address: &PublicKeyStr) -> String {
        let nonce = Uuid::new_v4().to_string();
        self.nonces.insert((*address).clone(), nonce.clone());
        nonce
    }

    /**
     * Adds new transaction to the blockchain.
     * The transaction has to be signed
     */
    pub fn append_blockchain(
        &mut self,
        signed_transaction: Transaction,
    ) -> Result<()> {
        signed_transaction.verify_transaction()?;
        self.ensure_nonce_exists_and_valid(
            &signed_transaction.from_address,
            &signed_transaction.nonce,
        )?;
        let balance = self.balance_for_address(&signed_transaction.from_address)?;
        if balance < signed_transaction.amount {
            return Err(anyhow::anyhow!(
                "Address {} has insufficient funds: {}",
                &signed_transaction.from_address,
                balance
            ));
        }

        let tip_hash = if self.blocks.is_empty() {
            "0".to_string()
        } else {
            self.blocks[self.blocks.len() - 1].hash.to_owned()
        };

        let mut hasher = Sha256::new();
        hasher.update(tip_hash.as_bytes());
        hasher.update(signed_transaction.signed_transaction_sha256_hash());
        let next_hash = hasher.finalize().to_vec();

        // remove the nonce
        self.nonces.remove(&signed_transaction.from_address);

        let block = Block {
            prev_hash: tip_hash,
            hash: hex::encode(next_hash),
            transaction: signed_transaction,
        };

        self.blocks.push(block);

        Ok(())
    }

    pub fn ensure_nonce_exists_and_valid(&self, address: &PublicKeyStr, nonce: &str) -> Result<()> {
        let existing_nonce = self.nonces.get(address).ok_or(anyhow::anyhow!(
            "Nonce does not exist for address {}",
            address
        ))?;
        if existing_nonce != nonce {
            return Err(anyhow::anyhow!(
                "Provided nonce does not match existing nonce: {} != {}",
                nonce,
                existing_nonce
            ));
        }
        Ok(())
    }

    pub fn hash(&self) -> Option<String> {
        self.blocks.last().map(|b| b.hash.clone())
    }

    /**
     * Computes the hash of the last transaction and verifies each transaction in the process
     */
    pub fn validate_blockchain(&self) -> Result<String> {
        let hash: Result<String> =
            self.blocks
                .iter()
                .try_fold("0".to_string(), |last_hash, block| {
                    let transaction = &block.transaction;
                    transaction.verify_transaction()?;
                    let block_hash = Block::calculate_block_hash(&last_hash, &block.transaction)?;

                    if block_hash != block.hash {
                        return Err(anyhow::anyhow!(
                            "Existing block hash does not match to transaction hash: {} vs {}",
                            block_hash,
                            block.hash
                        ));
                    }

                    Ok(block_hash)
                });

        hash
    }
}