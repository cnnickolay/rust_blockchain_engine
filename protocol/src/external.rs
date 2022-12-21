use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct ExternalRequest {
    pub request_id: String,
    pub command: UserCommand
}

#[derive(Serialize, Deserialize, Debug)]
pub enum UserCommand {
    PingCommand {
        msg: String
    },
    CreateRecord {
        data: String
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub enum UserCommandResponse {
    PingCommandResponse {
        request_id: String,
        msg: String
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ExternalResponse {
    Success(UserCommandResponse),
    Error {
        msg: String
    }
}
