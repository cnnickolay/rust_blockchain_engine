use rsa::{RsaPrivateKey, RsaPublicKey};
use anyhow::Result;
use sha1::Digest;
use sha2::Sha256;
use super::transaction::Transaction;

pub struct Block {
    pub prev_hash: String,
    /**
     * Hash combines hashes of both transaction and transaction_signature
     */
    pub hash: String,
    pub transaction: Transaction,
}

impl Block {
    pub fn new_signed(
        tip_hash: &str,
        nonce: &str,
        private_key: &RsaPrivateKey,
        from_address: &RsaPublicKey,
        to_address: &RsaPublicKey,
        amount: u64,
    ) -> Result<Self> {
        let transaction = Transaction::new_unsigned(
            nonce.to_string(),
            from_address.try_into()?,
            to_address.try_into()?,
            amount,
        )
        .sign(private_key)?;

        let next_hash = Block::calculate_block_hash(&tip_hash, &transaction)?;

        let block = Block {
            prev_hash: tip_hash.to_owned(),
            hash: next_hash,
            transaction,
        };

        Ok(block)
    }

    pub fn calculate_block_hash(last_hash: &str, transaction: &Transaction) -> Result<String> {
        let mut hasher = Sha256::new();
        hasher.update(last_hash.as_bytes());
        hasher.update(transaction.signed_transaction_sha256_hash());
        let next_hash = String::from_utf8(Sha256::digest(hasher.finalize()).to_vec())?;
        Ok(next_hash)
    }
}