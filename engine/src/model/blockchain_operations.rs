use super::model::{BlockChain, Transaction};
use sha2::{Sha256, Digest};


fn compute_hash(blockchain: &BlockChain) -> String {
    let hash = blockchain.transactions.iter().fold("0".to_string(), |result, transaction| {
        let mut hasher = Sha256::new();
        hasher.update(result.as_bytes());
        hasher.update(transaction.from_address.0.as_bytes());
        hasher.update(transaction.to_address.0.as_bytes());
        hasher.update(transaction.amount.to_le_bytes());
        hasher.update(transaction.signature.0.as_bytes());
        let hash = hasher.finalize();
        hex::encode(&hash[..])
    });

    hash
}

/**
 * Verifies the legitimacy of a transaction by checking its signature
 */
fn verify_transaction(transaction: &Transaction) -> bool {
    
    false
}