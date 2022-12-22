use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum InternalRequest {
    RegisterValidator {
        request_id: String,
        destination: String // e.g. 127.0.0.1:8080
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub enum InternalResponse {
    RegisterValidator {
        request_id: String
    }
}

//

impl InternalRequest {
    pub fn hash(&self) -> &String {
        match self {
            InternalRequest::RegisterValidator { request_id: hash, .. } => hash,
        }
    }
}

impl InternalResponse {
    pub fn hash(&self) -> &String {
        match self {
            InternalResponse::RegisterValidator { request_id: hash, .. } => hash,
        }
    }
}
