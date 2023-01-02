use sha1::Digest;
use sha2::Sha256;

pub struct Cbor(pub String);

impl Cbor {
    pub fn new(cbor: &str) -> Self {
        Cbor(cbor.to_owned())
    }

    pub fn hash(&self) -> Vec<u8> {
        let digest = Sha256::digest(&self.0);
        digest.to_vec()
    }
}