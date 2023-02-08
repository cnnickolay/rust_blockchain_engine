use anyhow::Result;
use protocol::request::Response;

pub trait Serializer {
    fn serialize(&self) -> Result<Vec<u8>>;
}

impl Serializer for Response {
    fn serialize(&self) -> Result<Vec<u8>> {
        Ok(serde_cbor::to_vec(&self)?)
    }
}
