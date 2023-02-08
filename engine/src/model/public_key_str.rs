use std::fmt::{Display, Formatter};

use rsa::{
    pkcs1::{DecodeRsaPublicKey, EncodeRsaPublicKey},
    RsaPublicKey,
};
use serde::{Deserialize, Serialize};

use crate::utils::shorten_long_string;

use super::hex_string::HexString;

#[derive(Clone, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub struct PublicKeyStr(pub HexString);

impl Display for PublicKeyStr {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", shorten_long_string(&self.0 .0))
    }
}

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

impl From<&PublicKeyStr> for String {
    fn from(pk: &PublicKeyStr) -> Self {
        pk.0 .0.to_owned()
    }
}
