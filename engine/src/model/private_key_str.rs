use rsa::RsaPrivateKey;

use super::HexString;

#[derive(Clone)]
pub struct PrivateKeyStr(pub String);

impl PrivateKeyStr {
    pub fn from_str(str: &str) -> Self {
        PrivateKeyStr(str.to_owned())
    }
}

impl TryFrom<&PrivateKeyStr> for RsaPrivateKey {
    type Error = anyhow::Error;

    fn try_from(value: &PrivateKeyStr) -> Result<Self, Self::Error> {
        (&HexString(value.0.clone())).try_into()
    }
}

impl TryFrom<&RsaPrivateKey> for PrivateKeyStr {
    type Error = anyhow::Error;

    fn try_from(value: &RsaPrivateKey) -> Result<Self, Self::Error> {
        let hex: HexString = value.try_into()?;
        Ok(PrivateKeyStr(hex.0))
    }
}