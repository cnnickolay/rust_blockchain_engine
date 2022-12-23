use serde::{Serialize, Deserialize};
use uuid::Uuid;

use crate::request::Request;


#[derive(Serialize, Deserialize, Debug)]
pub struct ExternalRequest {
    pub request_id: String,
    pub command: UserCommand
}

impl ExternalRequest {
    pub fn new(command: UserCommand) -> Self {
        let request_id = Uuid::new_v4().to_string();
        ExternalRequest {
            request_id,
            command
        }
    }
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

impl UserCommand {
    pub fn new_ping(msg: &str) -> UserCommand {
        UserCommand::PingCommand { msg: msg.to_string() }
    }

    pub fn to_request(self) -> Request {
        Request::External(ExternalRequest::new(self))
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
