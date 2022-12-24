use sha1::Digest;
use sha2::Sha256;

use super::model::{BlockChain, PublicKey, Signature, Transaction};

impl BlockChain {
    pub fn new() -> Self {
        BlockChain {
            transactions: Vec::new(),
        }
    }
}

impl Transaction {
    pub fn new(from_address: &str, to_address: &str, amount: u64, signature: &str) -> Self {
        Transaction {
            from_address: PublicKey(from_address.to_owned()),
            to_address: PublicKey(to_address.to_owned()),
            amount,
            signature: Signature(signature.to_owned()),
        }
    }

    pub fn to_sha256_hash(&self) -> Vec<u8> {
        let mut transaction_str = String::new();
        transaction_str.push_str(&self.from_address.0);
        transaction_str.push_str(&self.to_address.0);
        transaction_str.push_str(&self.amount.to_string());
        Sha256::digest(transaction_str).to_vec()
    }
}
