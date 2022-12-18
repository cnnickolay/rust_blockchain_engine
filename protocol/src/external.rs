use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct ExternalRequest {
    pub request_id: String,
    pub command: UserCommand
}

#[derive(Serialize, Deserialize, Debug)]
pub enum UserCommand {
    CreateRecord {
        data: String
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ExternalResponse {
    Success(),
    Error {
        msg: String
    }
}
