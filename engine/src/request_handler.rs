use protocol::{internal::{InternalResponse, InternalRequest, CommandResponse}, external::{ExternalResponse, ExternalRequest, UserCommand, UserCommandResponse}};
use serde::Serialize;

use crate::{configuration::{Configuration, NodeType, ValidatorAddress}, model::State};

pub trait RequestHandler<T : Serialize> {
    type RESPONSE;
    fn handle_request(&self, configuration: &mut Configuration, state: &mut State) -> Self::RESPONSE;
}

impl RequestHandler<InternalResponse> for InternalRequest {
    type RESPONSE = InternalResponse;
    fn handle_request(&self, configuration: &mut Configuration, state: &mut State) -> Self::RESPONSE {
        match &self.command {
            protocol::internal::CommandRequest::OnBoardValidator { return_address } => {
                let mut validators: Vec<ValidatorAddress> = match &configuration.node_type {
                    NodeType::Coordinator { validators } => validators.clone(),
                    NodeType::Validator => panic!("Validator can't on-board another validator"),
                };

                let validator_address = ValidatorAddress(return_address.to_owned());
                &validators.push(validator_address.clone());
                let total_validators = validators.len();
                let node_type = NodeType::Coordinator { validators };
                configuration.node_type = node_type;

                println!("Added new validator {:?}, total validators {}", &validator_address, total_validators);

                InternalResponse::Success(CommandResponse::OnBoardValidatorResponse)
            },
        }
    }
}

impl RequestHandler<ExternalResponse> for ExternalRequest {
    type RESPONSE = ExternalResponse;
    fn handle_request(&self, configuration: &mut Configuration, state: &mut State) -> Self::RESPONSE {
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
