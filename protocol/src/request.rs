use crate::{internal::{InternalRequest}, external::{ExternalRequest}};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum Request {
    Internal (InternalRequest),
    External (ExternalRequest)
}

impl Request {
    pub fn request_id(&self) -> String {
        match self {
            Request::Internal(req) => req.request_id.to_owned(),
            Request::External(req) => req.request_id.to_owned(),
        }
    }
}