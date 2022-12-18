use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum InternalRequest {
    Connect {
        request_id: String,
        address: String // e.g. 127.0.0.1:8080
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub enum InternalResponse {
    Connect {
        request_id: String
    }
}

//

impl InternalRequest {
    pub fn hash(&self) -> &String {
        match self {
            InternalRequest::Connect { request_id: hash, .. } => hash,
        }
    }
}

impl InternalResponse {
    pub fn hash(&self) -> &String {
        match self {
            InternalResponse::Connect { request_id: hash, .. } => hash,
        }
    }
}