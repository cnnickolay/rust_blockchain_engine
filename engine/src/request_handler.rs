use protocol::{internal::{InternalResponse, InternalRequest}, external::{ExternalResponse, ExternalRequest, UserCommand, UserCommandResponse}};
use serde::Serialize;

pub trait RequestHandler<T : Serialize> {
    type RESPONSE;
    fn handle_request(&self) -> Self::RESPONSE;
}

impl RequestHandler<InternalResponse> for InternalRequest {
    type RESPONSE = InternalResponse;
    fn handle_request(&self) -> Self::RESPONSE {
        todo!()
    }
}

impl RequestHandler<ExternalResponse> for ExternalRequest {
    type RESPONSE = ExternalResponse;
    fn handle_request(&self) -> Self::RESPONSE {
        dbg!("External request received");
        match &self.command {
            UserCommand::CreateRecord { data } => panic!("Not ready yet"),
            UserCommand::PingCommand { msg } => {
                dbg!("Received ping command");
                ExternalResponse::Success(UserCommandResponse::PingCommandResponse { 
                    request_id: self.request_id.clone(), 
                    msg: format!("Original message: {}, PONG PONG", msg) 
                })
            },
        }
    }
}
