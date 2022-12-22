use rsa::{PublicKey, RsaPrivateKey, RsaPublicKey, PaddingScheme};
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