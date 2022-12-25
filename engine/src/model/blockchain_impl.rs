use super::blockchain::{Block, BlockChain, HexString, PublicKeyStr, Signature, Transaction};
use anyhow::Result;
use rsa::{PaddingScheme, PublicKey, RsaPrivateKey, RsaPublicKey};
use sha1::Digest;
use sha2::Sha256;

impl BlockChain {
    pub fn new() -> Self {
        BlockChain { blocks: Vec::new() }
    }

    pub fn from_vector(signed_transactions: &[(Transaction, Signature)]) -> Result<Self> {
        let mut blockchain = BlockChain::new();

        for (transaction, signature) in signed_transactions {
            blockchain.append_blockchain((*transaction).clone(), (*signature).clone())?;
        }

        Ok(blockchain)
    }

    /**
     * Adds new transaction to the blockchain.
     * The transaction has to be signed
     */
    pub fn append_blockchain(
        &mut self,
        transaction: Transaction,
        signature: Signature,
    ) -> Result<()> {
        transaction.verify_transaction(&signature)?;

        let tip_hash = if self.blocks.is_empty() {
            "0".to_string()
        } else {
            self.blocks[self.blocks.len() - 1].hash.to_owned()
        };

        let mut hasher = Sha256::new();
        hasher.update(tip_hash.as_bytes());
        hasher.update(transaction.to_sha256_hash());
        let next_hash = hasher.finalize().to_vec();

        let block = Block {
            prev_hash: tip_hash,
            hash: hex::encode(next_hash),
            transaction,
            transaction_signature: signature,
        };

        self.blocks.push(block);

        Ok(())
    }

    pub fn hash(&self) -> Option<String> {
        self.blocks.last().map(|b| b.hash.clone())
    }

    /**
     * Computes the hash of the last transaction and verifies each transaction in the process
     */
    pub fn validate_blockchain(&self) -> Result<String> {
        let hash: Result<String> = self
            .blocks
            .iter()
            .try_fold("0".to_string(), |last_hash, block| {
                let transaction = &block.transaction;
                transaction.verify_transaction(&block.transaction_signature)?;
                let block_hash = Block::calculate_block_hash(&last_hash, &block.transaction, &block.transaction_signature)?;

                if block_hash != block.hash {
                    return Err(anyhow::anyhow!("Existing block hash does not match to transaction hash: {} vs {}", block_hash, block.hash));
                }

                Ok(block_hash)
            });

        hash
    }
}

impl Block {
    pub fn new_signed(
        tip_hash: &str,
        private_key: &RsaPrivateKey,
        from_address: &RsaPublicKey,
        to_address: &RsaPublicKey,
        amount: u64,
    ) -> Result<Self> {
        let transaction =
            Transaction::new(from_address.try_into()?, to_address.try_into()?, amount);
        let hash = &transaction.to_sha256_hash();
        let signature = Signature::sign(private_key, hash)?;

        let next_hash = Block::calculate_block_hash(&tip_hash, &transaction, &signature)?;

        let block = Block {
            prev_hash: tip_hash.to_owned(),
            hash: next_hash,
            transaction,
            transaction_signature: signature,
        };

        Ok(block)
    }

    pub fn calculate_block_hash(
        last_hash: &str,
        transaction: &Transaction,
        signature: &Signature,
    ) -> Result<String> {
        let mut hasher = Sha256::new();
        hasher.update(last_hash.as_bytes());
        hasher.update(transaction.to_sha256_hash());
        hasher.update(signature.0 .0.as_bytes());
        let next_hash = String::from_utf8(Sha256::digest(hasher.finalize()).to_vec())?;
        Ok(next_hash)
    }
}

impl Transaction {
    pub fn new(from_address: PublicKeyStr, to_address: PublicKeyStr, amount: u64) -> Self {
        Transaction {
            from_address,
            to_address,
            amount,
        }
    }

    pub fn to_sha256_hash(&self) -> Vec<u8> {
        let mut hasher = Sha256::new();
        hasher.update(self.from_address.0 .0.as_bytes());
        hasher.update(self.to_address.0 .0.as_bytes());
        hasher.update(self.amount.to_ne_bytes());
        hasher.finalize().to_vec()
    }

    /**
     * Verifies the legitimacy of a transaction by checking its signature
     */
    pub fn verify_transaction(&self, signature: &Signature) -> Result<()> {
        let public_key = RsaPublicKey::try_from(&self.from_address)?;
        let digest = self.to_sha256_hash();
        signature.verify(&public_key, &digest)?;
        Ok(())
    }
}

impl Signature {
    pub fn sign(private_key: &RsaPrivateKey, digest: &[u8]) -> Result<Signature> {
        let signature =
            hex::encode(private_key.sign(PaddingScheme::new_pkcs1v15_sign::<Sha256>(), &digest)?);
        Ok(Signature(HexString(signature)))
    }

    pub fn verify(&self, public_key: &RsaPublicKey, digest: &[u8]) -> Result<()> {
        let padding = PaddingScheme::new_pkcs1v15_sign::<Sha256>();
        let signature = hex::decode(&self.0 .0)?;
        public_key.verify(
            PaddingScheme::new_pkcs1v15_sign::<Sha256>(),
            &digest,
            &signature,
        )?;
        Ok(())
    }

    pub fn empty() -> Self {
        Signature(HexString(String::new()))
    }
}

impl PublicKeyStr {
    pub fn from_str(str: &str) -> Self {
        PublicKeyStr(HexString(str.to_owned()))
    }
}
