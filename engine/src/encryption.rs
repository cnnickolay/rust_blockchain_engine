use anyhow::Result;
use rsa::{RsaPrivateKey, RsaPublicKey};

use crate::model::{PrivateKeyStr, PublicKeyStr};

pub fn generate_rsa_key_pair() -> Result<(RsaPrivateKey, RsaPublicKey)> {
    let mut rng = rand::thread_rng();
    let bits = 2048;
    let from_priv_key = RsaPrivateKey::new(&mut rng, bits)?;
    let from_pub_key = RsaPublicKey::from(&from_priv_key);
    Ok((from_priv_key, from_pub_key))
}

pub fn generate_rsa_keypair_custom() -> Result<(PrivateKeyStr, PublicKeyStr)> {
    let (priv_key, pub_key) = generate_rsa_key_pair()?;
    let priv_key_str = PrivateKeyStr::try_from(&priv_key).unwrap();
    let pub_key_str = PublicKeyStr::try_from(&pub_key).unwrap();
    Ok((priv_key_str, pub_key_str))
}
