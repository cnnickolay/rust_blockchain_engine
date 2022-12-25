use super::blockchain::{BlockChain, HexString, PublicKeyStr, Signature, Transaction};
use anyhow::Result;
use rsa::{PaddingScheme, PublicKey, RsaPrivateKey, RsaPublicKey};
use sha1::Digest;
use sha2::Sha256;

impl BlockChain {
    pub fn new() -> Self {
        BlockChain {
            transactions: Vec::new(),
        }
    }

    pub fn from_vector(transactions: Vec<Transaction>) -> Self {
        BlockChain {
            transactions,
        }
    }

    pub fn add_transaction(&mut self, transaction: Transaction) {
        self.transactions.push(transaction);
    }

    pub fn compute_hash(&self) -> String {
        let hash = self
            .transactions
            .iter()
            .fold("0".to_string(), |result, transaction| {
                let mut hasher = Sha256::new();
                hasher.update(result.as_bytes());
                hasher.update(transaction.from_address.0 .0.as_bytes());
                hasher.update(transaction.to_address.0 .0.as_bytes());
                hasher.update(transaction.amount.to_le_bytes());
                hasher.update(transaction.signature.0 .0.as_bytes());
                let hash = hasher.finalize();
                hex::encode(&hash[..])
            });
    
        hash
    }
}

impl Transaction {
    pub fn new(
        from_address: PublicKeyStr,
        to_address: PublicKeyStr,
        amount: u64,
        signature: Signature,
    ) -> Self {
        Transaction {
            from_address: from_address,
            to_address: to_address,
            amount,
            signature: signature,
        }
    }

    pub fn to_sha256_hash_bytes(&self) -> Vec<u8> {
        let mut transaction_str = String::new();
        transaction_str.push_str(&self.from_address.0 .0);
        transaction_str.push_str(&self.to_address.0 .0);
        transaction_str.push_str(&self.amount.to_string());
        Sha256::digest(transaction_str).to_vec()
    }

    /**
     * Verifies the legitimacy of a transaction by checking its signature
     */
    pub fn verify_transaction(&self) -> Result<()> {
        let public_key = RsaPublicKey::try_from(&self.from_address)?;
        let digest = self.to_sha256_hash_bytes();
        self.signature.verify(&public_key, &digest)?;
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