use rsa::{PublicKey, RsaPrivateKey, RsaPublicKey, PaddingScheme, pkcs1::{DecodeRsaPublicKey, EncodeRsaPublicKey}, pkcs8::LineEnding};
use sha1::{Sha1, Digest};


pub fn encrypt() {
    println!("started");
    let mut rng = rand::thread_rng();
    let bits = 2048;
    let priv_key = RsaPrivateKey::new(&mut rng, bits).expect("failed to generate a key");
    let pub_key = RsaPublicKey::from(&priv_key);


    let digest = Sha1::digest(b"hello world").to_vec();

    let signature = priv_key.sign(PaddingScheme::new_pkcs1v15_sign::<Sha1>(), &digest).unwrap();
    let result = pub_key.verify(PaddingScheme::new_pkcs1v15_sign::<Sha1>(), &digest, &signature).unwrap();

    // let pem = pub_key.to_pkcs1_pem(LineEnding::LF).unwrap();
    let pem = hex::encode(pub_key.to_pkcs1_der().unwrap());
    // let pem = String::from_utf8(pub_key.to_pkcs1_der().unwrap().as_bytes().to_vec()).unwrap();
    println!("pem: {}", pem);
    println!("signature: {}", hex::encode(&signature));

    // let pub_key_restored = RsaPublicKey::from_pkcs1_pem(&pem).unwrap();
    let pub_key_restored = RsaPublicKey::from_pkcs1_der(&hex::decode(pem).unwrap()).unwrap();
    let result = pub_key_restored.verify(PaddingScheme::new_pkcs1v15_sign::<Sha1>(), &digest, &signature).unwrap();

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