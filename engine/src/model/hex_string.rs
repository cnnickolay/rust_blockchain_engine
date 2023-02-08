use rsa::{RsaPrivateKey, RsaPublicKey};
use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub struct HexString(pub String);

impl TryFrom<&RsaPublicKey> for HexString {
    type Error = anyhow::Error;

    fn try_from(value: &RsaPublicKey) -> Result<Self, Self::Error> {
        use rsa::pkcs1::EncodeRsaPublicKey;
        let key_str = hex::encode(value.to_pkcs1_der()?);
        Ok(HexString(key_str))
    }
}

impl TryFrom<&RsaPrivateKey> for HexString {
    type Error = anyhow::Error;

    fn try_from(value: &RsaPrivateKey) -> Result<Self, Self::Error> {
        use rsa::pkcs1::EncodeRsaPrivateKey;
        let key_str = hex::encode(value.to_pkcs1_der()?.as_bytes());
        Ok(HexString(key_str))
    }
}

impl TryFrom<&HexString> for RsaPublicKey {
    type Error = anyhow::Error;

    fn try_from(value: &HexString) -> Result<Self, Self::Error> {
        use rsa::pkcs1::DecodeRsaPublicKey;
        let key_bytes = hex::decode(&value.0)?;
        let key = RsaPublicKey::from_pkcs1_der(&key_bytes)?;
        Ok(key)
    }
}

impl TryFrom<&HexString> for RsaPrivateKey {
    type Error = anyhow::Error;

    fn try_from(value: &HexString) -> Result<Self, Self::Error> {
        use rsa::pkcs1::DecodeRsaPrivateKey;
        let key_bytes = hex::decode(&value.0)?;
        let key = RsaPrivateKey::from_pkcs1_der(&key_bytes)?;
        Ok(key)
    }
}
