use super::model::{BlockChain, Signature, Transaction};
use anyhow::Result;
use rsa::{
    pkcs1::{DecodeRsaPublicKey, EncodeRsaPublicKey},
    pkcs8::der::Encode,
    PaddingScheme, PublicKey, RsaPrivateKey, RsaPublicKey,
};
use sha1::Sha1;
use sha2::{Digest, Sha256};

fn compute_hash(blockchain: &BlockChain) -> String {
    let hash = blockchain
        .transactions
        .iter()
        .fold("0".to_string(), |result, transaction| {
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
fn verify_transaction(transaction: &Transaction) -> Result<()> {
    let pub_key_str = hex::decode(&transaction.from_address.0)?;
    let pub_key_restored = RsaPublicKey::from_pkcs1_der(&pub_key_str)?;
    let digest = transaction.to_sha256_hash();
    pub_key_restored.verify(
        PaddingScheme::new_pkcs1v15_sign::<Sha256>(),
        &digest,
        &hex::decode(&transaction.signature.0).unwrap(),
    )?;
    Ok(())
}

#[test]
fn verify_transaction_test_success() {
    let mut rng = rand::thread_rng();
    let bits = 2048;
    let from_priv_key = RsaPrivateKey::new(&mut rng, bits).expect("failed to generate a key");
    let from_pub_key = RsaPublicKey::from(&from_priv_key);
    let to_priv_key = RsaPrivateKey::new(&mut rng, bits).expect("failed to generate a key");
    let to_pub_key = RsaPublicKey::from(&to_priv_key);

    let from_address = hex::encode(from_pub_key.to_pkcs1_der().unwrap());
    let to_address = hex::encode(to_pub_key.to_pkcs1_der().unwrap());
    let mut transaction = Transaction::new(&from_address, &to_address, 100, &"");
    let digest = transaction.to_sha256_hash();

    let signature = hex::encode(
        from_priv_key
            .sign(PaddingScheme::new_pkcs1v15_sign::<Sha256>(), &digest)
            .unwrap(),
    );
    transaction.signature = Signature(signature);

    verify_transaction(&transaction).unwrap();
}

pub fn encrypt() {
    println!("started");
    let mut rng = rand::thread_rng();
    let bits = 2048;
    let priv_key = RsaPrivateKey::new(&mut rng, bits).expect("failed to generate a key");
    let pub_key = RsaPublicKey::from(&priv_key);

    let digest = Sha1::digest(b"hello world").to_vec();

    let signature = priv_key
        .sign(PaddingScheme::new_pkcs1v15_sign::<Sha1>(), &digest)
        .unwrap();
    let result = pub_key
        .verify(
            PaddingScheme::new_pkcs1v15_sign::<Sha1>(),
            &digest,
            &signature,
        )
        .unwrap();

    // let pem = pub_key.to_pkcs1_pem(LineEnding::LF).unwrap();
    let pem = hex::encode(pub_key.to_pkcs1_der().unwrap());
    // let pem = String::from_utf8(pub_key.to_pkcs1_der().unwrap().as_bytes().to_vec()).unwrap();
    println!("pem: {}", pem);
    println!("signature: {}", hex::encode(&signature));

    // let pub_key_restored = RsaPublicKey::from_pkcs1_pem(&pem).unwrap();
    let pub_key_restored = RsaPublicKey::from_pkcs1_der(&hex::decode(pem).unwrap()).unwrap();
    let result = pub_key_restored
        .verify(
            PaddingScheme::new_pkcs1v15_sign::<Sha1>(),
            &digest,
            &signature,
        )
        .unwrap();

    // println!("encrypting");
    // // Encrypt
    // let enc_data = pub_key.encrypt(&mut rng, PaddingScheme::new_pkcs1v15_encrypt(), &data[..]).expect("failed to encrypt");
    // assert_ne!(&data[..], &enc_data[..]);

    // println!("decrypting");
    // // Decrypt
    // let dec_data = priv_key.decrypt(PaddingScheme::new_pkcs1v15_encrypt(), &enc_data).expect("failed to decrypt");
    // assert_eq!(&data[..], &dec_data[..]);
    // println!("finished");
}

#[test]
fn encrypt_test() {
    encrypt()
}
