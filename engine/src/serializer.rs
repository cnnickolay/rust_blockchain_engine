use anyhow::Result;
use protocol::{internal::InternalResponse, external::ExternalResponse, response::Response};

pub trait Serializer {
    fn serialize(&self) -> Result<Vec<u8>>;
}

impl Serializer for InternalResponse {
    fn serialize(&self) -> Result<Vec<u8>> {
        Ok(serde_cbor::to_vec(&self)?)
    }
}

impl Serializer for ExternalResponse {
    fn serialize(&self) -> Result<Vec<u8>> {
        Ok(serde_cbor::to_vec(&self)?)
    }
}

impl Serializer for Response {
    fn serialize(&self) -> Result<Vec<u8>> {
        match self {
            Response::Internal(internal) => internal.serialize(),
            Response::External(external) => external.serialize(),
        }
    }
}