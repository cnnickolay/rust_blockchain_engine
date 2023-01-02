use crate::{external::ExternalResponse, internal::InternalResponse};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum Response {
    Internal(InternalResponse),
    External(ExternalResponse),
}

impl Response {
    pub fn external_response(&self) -> Option<&ExternalResponse> {
        match self {
            Response::External(external) => Some(external),
            _ => None,
        }
    }
    pub fn internal_response(&self) -> Option<&InternalResponse> {
        match self {
            Response::Internal(internal) => Some(internal),
            _ => None,
        }
    }
}
