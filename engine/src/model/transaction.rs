use rsa::{RsaPrivateKey, RsaPublicKey};
use anyhow::Result;
use sha1::Digest;
use sha2::Sha256;
use super::{public_key_str::PublicKeyStr, signature::Signature};

#[derive(Clone, PartialEq)]
pub struct Transaction {
    pub nonce: String,
    pub from_address: PublicKeyStr,
    pub to_address: PublicKeyStr,
    pub amount: u64,
    pub signature: Signature,
}

impl Transaction {
    pub fn new_unsigned(
        nonce: String,
        from_address: PublicKeyStr,
        to_address: PublicKeyStr,
        amount: u64,
    ) -> Self {
        Transaction {
            nonce,
            from_address,
            to_address,
            amount,
            signature: Signature::empty(),
        }
    }

    pub fn sign(mut self, private_key: &RsaPrivateKey) -> Result<Self> {
        self.signature = Signature::sign(private_key, &self.sha256_hash_for_signing())?;
        Ok(self)
    }

    pub fn signed_transaction_sha256_hash(&self) -> Vec<u8> {
        let mut hasher = Sha256::new();
        hasher.update(self.nonce.as_bytes());
        hasher.update(self.from_address.0 .0.as_bytes());
        hasher.update(self.to_address.0 .0.as_bytes());
        hasher.update(self.amount.to_ne_bytes());
        hasher.update(self.signature.0 .0.as_bytes());
        hasher.finalize().to_vec()
    }

    fn sha256_hash_for_signing(&self) -> Vec<u8> {
        let mut hasher = Sha256::new();
        hasher.update(self.nonce.as_bytes());
        hasher.update(self.from_address.0 .0.as_bytes());
        hasher.update(self.to_address.0 .0.as_bytes());
        hasher.update(self.amount.to_ne_bytes());
        hasher.finalize().to_vec()
    }

    /**
     * Verifies the legitimacy of a transaction by checking its signature
     */
    pub fn verify_transaction(&self) -> Result<()> {
        let public_key = RsaPublicKey::try_from(&self.from_address)?;
        let digest = self.sha256_hash_for_signing();

        self.signature.verify(&public_key, &digest)?;
        Ok(())
    }
}