use rsa::{RsaPrivateKey, PaddingScheme, RsaPublicKey, PublicKey};
use anyhow::Result;
use serde::{Serialize, Deserialize};
use sha1::Digest;
use sha2::Sha256;
use super::hex_string::HexString;

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize, Eq, Hash)]
pub struct Signature(pub HexString);

impl Signature {
    pub fn sign(private_key: &RsaPrivateKey, cbor: &[u8]) -> Result<Signature> {
        let digest = Sha256::digest(cbor);
        let signature_bytes = private_key.sign(PaddingScheme::new_pkcs1v15_sign::<Sha256>(), &digest)?;
        let signature = hex::encode(signature_bytes);
        Ok(Signature(HexString(signature)))
    }

    pub fn verify(&self, public_key: &RsaPublicKey, cbor: &[u8]) -> Result<()> {
        let digest = Sha256::digest(cbor);
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