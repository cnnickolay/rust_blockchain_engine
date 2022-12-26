use super::blockchain::{HexString, PublicKeyStr};
use anyhow::Result;
use rsa::{
    pkcs1::{DecodeRsaPrivateKey, DecodeRsaPublicKey, EncodeRsaPublicKey},
    RsaPrivateKey, RsaPublicKey,
};

impl TryFrom<&PublicKeyStr> for RsaPrivateKey {
    type Error = anyhow::Error;

    fn try_from(value: &PublicKeyStr) -> Result<Self, Self::Error> {
        let key_bytes = hex::decode(&value.0 .0)?;
        let key = RsaPrivateKey::from_pkcs1_der(&key_bytes)?;
        Ok(key)
    }
}

impl TryFrom<&PublicKeyStr> for RsaPublicKey {
    type Error = anyhow::Error;

    fn try_from(value: &PublicKeyStr) -> Result<Self, Self::Error> {
        let key_bytes = hex::decode(&value.0 .0)?;
        let key = RsaPublicKey::from_pkcs1_der(&key_bytes)?;
        Ok(key)
    }
}

impl TryFrom<&RsaPrivateKey> for PublicKeyStr {
    type Error = anyhow::Error;

    fn try_from(key: &RsaPrivateKey) -> Result<Self, Self::Error> {
        let key_str = hex::encode(key.to_pkcs1_der()?);
        Ok(PublicKeyStr(HexString(key_str)))
    }
}

impl TryFrom<&RsaPublicKey> for PublicKeyStr {
    type Error = anyhow::Error;

    fn try_from(key: &RsaPublicKey) -> Result<Self, Self::Error> {
        let key_str = hex::encode(key.to_pkcs1_der()?);
        Ok(PublicKeyStr(HexString(key_str)))
    }
}

fn generate_rsa_key_pair() -> Result<(RsaPrivateKey, RsaPublicKey)> {
    let mut rng = rand::thread_rng();
    let bits = 2048;
    let from_priv_key = RsaPrivateKey::new(&mut rng, bits)?;
    let from_pub_key = RsaPublicKey::from(&from_priv_key);
    Ok((from_priv_key, from_pub_key))
}

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use rsa::RsaPrivateKey;
    use rsa::RsaPublicKey;
    use sha1::Digest;
    use sha2::Sha256;

    use crate::model::blockchain::BlockChain;
    use crate::model::blockchain::PublicKeyStr;
    use crate::model::blockchain::Signature;
    use crate::model::blockchain::Transaction;

    use super::generate_rsa_key_pair;

    #[test]
    fn signing_test_success() -> Result<()> {
        let (private_key, public_key) = generate_rsa_key_pair()?;
        let digest = Sha256::digest(b"Hello world").to_vec();
        let signature = Signature::sign(&private_key, &digest)?;
        signature.verify(&public_key, &digest)?;
        Ok(())
    }

    #[test]
    fn signing_test_fail() -> Result<()> {
        let (private_key, public_key) = generate_rsa_key_pair()?;
        let digest = Sha256::digest(b"Hello world").to_vec();
        let wrong_digest = Sha256::digest(b"Hello space").to_vec();
        let signature = Signature::sign(&private_key, &digest)?;
        match signature.verify(&public_key, &wrong_digest) {
            Ok(_) => panic!("Test should fail"),
            Err(_) => Ok(()),
        }
    }

    #[test]
    fn verify_transaction_test_success() {
        let (from_priv_key, from_pub_key) =
            generate_rsa_key_pair().expect("Unable to generate keys");
        let (to_priv_key, to_pub_key) = generate_rsa_key_pair().expect("Unable to generate keys");

        let from_address = PublicKeyStr::try_from(&from_pub_key).unwrap();
        let to_address = PublicKeyStr::try_from(&to_pub_key).unwrap();
        let transaction = Transaction::new(from_address, to_address, 100);
        let digest = transaction.to_sha256_hash();

        let signature =
            Signature::sign(&from_priv_key, &digest).expect("Unable to sign the transaction");

        transaction
            .verify_transaction(&signature)
            .expect("Transaction should be verified successfully");
    }

    #[test]
    fn calculate_blockchain_hash_success() -> Result<()> {
        let (root_priv, root_pub) = &generate_rsa_key_pair()?;
        let (priv_1, pub_1) = &generate_rsa_key_pair()?;
        let (priv_2, pub_2) = &generate_rsa_key_pair()?;
        let (priv_3, pub_3) = &generate_rsa_key_pair()?;

        let transaction_with_signature = |private_key: &RsaPrivateKey, transaction: Transaction| {
            let signature = Signature::sign(private_key, &(&transaction).to_sha256_hash())?;
            Ok((transaction, signature)) as Result<(Transaction, Signature)>
        };
        let finance_address = |destination_address: &RsaPublicKey| {
            transaction_with_signature(
                root_priv,
                Transaction::new(root_pub.try_into()?, destination_address.try_into()?, 20),
            )
        };
        let transaction_1 = || {
            transaction_with_signature(
                priv_1,
                Transaction::new(pub_1.try_into()?, pub_2.try_into()?, 10),
            )
        };
        let transaction_2 = || {
            transaction_with_signature(
                priv_2,
                Transaction::new(pub_2.try_into()?, pub_3.try_into()?, 10),
            )
        };
        let transaction_3 = || {
            transaction_with_signature(
                priv_3,
                Transaction::new(pub_3.try_into()?, pub_1.try_into()?, 10),
            )
        };

        let blockchain_1_hash = BlockChain::from_vector(
            &root_pub.try_into()?,
            100,
            &[finance_address(pub_1)?, transaction_1()?, transaction_2()?],
        )?
        .hash()
        .unwrap();
        let blockchain_2_hash = BlockChain::from_vector(
            &root_pub.try_into()?,
            100,
            &[
                finance_address(pub_3)?,
                finance_address(pub_2)?,
                transaction_3()?,
                transaction_2()?,
            ],
        )?
        .hash()
        .unwrap();
        assert_ne!(blockchain_1_hash, blockchain_2_hash);

        let blockchain_1_hash_recalculated = BlockChain::from_vector(
            &root_pub.try_into()?,
            100,
            &[finance_address(pub_1)?, transaction_1()?, transaction_2()?],
        )?
        .hash()
        .unwrap();
        assert_eq!(blockchain_1_hash, blockchain_1_hash_recalculated);

        let blockchain_1_hash = BlockChain::from_vector(
            &root_pub.try_into()?,
            100,
            &[finance_address(pub_1)?, finance_address(pub_2)?, transaction_1()?, transaction_2()?],
        )?
        .hash()
        .unwrap();
        let blockchain_2_hash = BlockChain::from_vector(
            &root_pub.try_into()?,
            100,
            &[finance_address(pub_1)?, finance_address(pub_2)?, transaction_2()?],
        )?
        .hash()
        .unwrap();
        assert_ne!(blockchain_1_hash, blockchain_2_hash);

        Ok(())
    }
}
