#[cfg(test)]
mod tests {
    use anyhow::Result;
    use sha1::Digest;
    use sha2::Sha256;

    use crate::encryption::generate_rsa_key_pair;
    use crate::model::signature::Signature;
    use crate::model::HexString;

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
}
