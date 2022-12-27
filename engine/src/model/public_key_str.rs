use std::fmt::{Display, Formatter};

use rsa::{RsaPublicKey, pkcs1::{DecodeRsaPublicKey, EncodeRsaPublicKey}};

use super::hex_string::HexString;


#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct PublicKeyStr(pub HexString);

impl PublicKeyStr {
    pub fn from_str(str: &str) -> Self {
        PublicKeyStr(HexString(str.to_owned()))
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

impl TryFrom<&RsaPublicKey> for PublicKeyStr {
    type Error = anyhow::Error;

    fn try_from(key: &RsaPublicKey) -> Result<Self, Self::Error> {
        let key_str = hex::encode(key.to_pkcs1_der()?);
        Ok(PublicKeyStr(HexString(key_str)))
    }
}

impl Display for PublicKeyStr {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.0)
    }
}
