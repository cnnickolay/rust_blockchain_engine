#[cfg(test)]
mod tests {
    use anyhow::anyhow;
    use anyhow::Result;
    use sha1::Digest;
    use sha2::Sha256;

    use crate::encryption::generate_rsa_key_pair;
    use crate::model::HexString;
    use crate::model::blockchain::BlockChain;
    use crate::model::public_key_str::PublicKeyStr;
    use crate::model::signature::Signature;
    use crate::model::transaction::Transaction;

    #[test]
    fn signing_test_success() -> Result<()> {
        let (private_key, public_key) = generate_rsa_key_pair()?;
        let a = HexString::try_from(&private_key)?;
        let b = HexString::try_from(&public_key)?;
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
        let transaction =
            Transaction::new_unsigned("nonce".to_string(), from_address, to_address, 100)
                .sign(&from_priv_key)
                .unwrap();
        let digest = transaction.signed_transaction_sha256_hash();

        let signature =
            Signature::sign(&from_priv_key, &digest).expect("Unable to sign the transaction");

        transaction
            .verify_transaction()
            .expect("Transaction should be verified successfully");
    }

    #[test]
    fn transaction_commit_should_fail_if_nonce_does_not_exist() -> Result<()> {
        let (priv_1, pub_1) = &generate_rsa_key_pair()?;
        let (_, pub_2) = &generate_rsa_key_pair()?;
        let mut blockchain = BlockChain::new(&pub_1.try_into()?, 100);
        let transaction =
            Transaction::new_unsigned("0".to_string(), pub_1.try_into()?, pub_2.try_into()?, 10)
                .sign(&priv_1)?;
        match blockchain.append_blockchain(transaction) {
            Ok(_) => Err(anyhow!(
                "Nonce wasn't added, so transaction should not be committed"
            )),
            Err(_) => Ok(()),
        }
    }

    #[test]
    fn should_fail_when_committing_same_signed_transaction_twice() -> Result<()> {
        let (priv_1, pub_1) = &generate_rsa_key_pair()?;
        let (_, pub_2) = &generate_rsa_key_pair()?;
        let mut blockchain = BlockChain::new(&pub_1.try_into()?, 100);
        let nonce = blockchain.request_nonce_for_address(&pub_1.try_into()?);
        let transaction =
            Transaction::new_unsigned(nonce, pub_1.try_into()?, pub_2.try_into()?, 10)
                .sign(&priv_1)?;
        let same_transaction = transaction.clone();

        blockchain.append_blockchain(transaction)?; // this one is successful

        match blockchain.append_blockchain(same_transaction) {
            Ok(_) => Err(anyhow!(
                "Should not be able to commit same transaction twice"
            )),
            Err(_) => Ok(()),
        }
    }

    #[test]
    fn print_balances() -> Result<()> {
        let (priv_1, pub_1) = &generate_rsa_key_pair()?;
        let (_, pub_2) = &generate_rsa_key_pair()?;
        let mut blockchain = BlockChain::new(&pub_1.try_into()?, 100);

        assert_eq!(blockchain.all_balances().len(), 1);

        let nonce = blockchain.request_nonce_for_address(&pub_1.try_into()?);
        let transaction =
        Transaction::new_unsigned(nonce, pub_1.try_into()?, pub_2.try_into()?, 10)
            .sign(&priv_1)?;
        blockchain.append_blockchain(transaction)?; // this one is successful

        assert_eq!(blockchain.all_balances().len(), 2);

        Ok(())
    }
}
