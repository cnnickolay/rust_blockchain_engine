use serde::{Serialize, Deserialize};
use uuid::Uuid;

use crate::request::Request;

#[derive(Serialize, Deserialize, Debug)]
pub struct InternalRequest {
    pub request_id: String,
    pub command: CommandRequest
}

#[derive(Serialize, Deserialize, Debug)]
pub enum InternalResponse {
    Success(CommandResponse),
    Error {
        msg: String
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub enum CommandRequest {
    OnBoardValidator {
        return_address: String
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub enum CommandResponse {
    OnBoardValidatorResponse
}


// ################

impl CommandRequest {
    pub fn new_on_board_command(return_address: &str) -> CommandRequest {
        CommandRequest::OnBoardValidator { return_address: return_address.to_string() }
    }

    pub fn to_request(self) -> Request {
        Request::Internal(InternalRequest::new(self))
    }
}

impl InternalRequest {
    pub fn new(command: CommandRequest) -> Self {
        let request_id = Uuid::new_v4().to_string();
        InternalRequest {
            request_id,
            command
        }
    }
}