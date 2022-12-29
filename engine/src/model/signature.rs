use rsa::{RsaPrivateKey, PaddingScheme, RsaPublicKey, PublicKey};
use anyhow::Result;
use sha2::Sha256;
use super::hex_string::HexString;

#[derive(Clone, PartialEq, Debug)]
pub struct Signature(pub HexString);

impl Signature {
    pub fn sign(private_key: &RsaPrivateKey, digest: &[u8]) -> Result<Signature> {
        let signature =
            hex::encode(private_key.sign(PaddingScheme::new_pkcs1v15_sign::<Sha256>(), &digest)?);
        Ok(Signature(HexString(signature)))
    }

    pub fn verify(&self, public_key: &RsaPublicKey, digest: &[u8]) -> Result<()> {
        let padding = PaddingScheme::new_pkcs1v15_sign::<Sha256>();
        let signature = hex::decode(&self.0 .0)?;
        public_key.verify(
            PaddingScheme::new_pkcs1v15_sign::<Sha256>(),
            &digest,
            &signature,
        )?;
        Ok(())
    }

    pub fn empty() -> Self {
        Signature(HexString(String::new()))
    }

    pub fn from_string(s: &str) -> Self {
        Signature(HexString(s.to_string()))
    }
}