use protocol::{internal::{InternalResponse, InternalRequest, CommandResponse}, external::{ExternalResponse, ExternalRequest, UserCommand, UserCommandResponse}};
use serde::Serialize;

use crate::{configuration::{Configuration, NodeType, ValidatorAddress}};

pub trait RequestHandler<T : Serialize> {
    type RESPONSE;
    fn handle_request(&self, configuration: &mut Configuration) -> Self::RESPONSE;
}

impl RequestHandler<InternalResponse> for InternalRequest {
    type RESPONSE = InternalResponse;
    fn handle_request(&self, configuration: &mut Configuration) -> Self::RESPONSE {
        match &self.command {
            protocol::internal::CommandRequest::OnBoardValidator { return_address } => {
                match configuration.node_type {
                    NodeType::Coordinator { ref mut validators } => {
                        let validator_address = ValidatorAddress("123".to_string());
                        validators.push(validator_address);
                        println!("Added new validator {:?}, total validators {}", return_address, &validators.len());
                    },
                    NodeType::Validator => panic!("Validator can't on-board another validator"),
                };

                InternalResponse::Success(CommandResponse::OnBoardValidatorResponse)
            },
        }
    }
}

impl RequestHandler<ExternalResponse> for ExternalRequest {
    type RESPONSE = ExternalResponse;
    fn handle_request(&self, configuration: &mut Configuration) -> Self::RESPONSE {
        println!("External request received");
        match &self.command {
            UserCommand::CreateRecord { data } => panic!("Not ready yet"),
            UserCommand::PingCommand { msg } => {
                println!("Received ping command");
                ExternalResponse::Success(UserCommandResponse::PingCommandResponse { 
                    request_id: self.request_id.clone(), 
                    msg: format!("Original message: {}, PONG PONG", msg)
                })
            },
        }
    }
}
