use protocol::{
    external::{ExternalRequest, ExternalResponse, UserCommand, UserCommandResponse},
    internal::{CommandResponse, InternalRequest, InternalResponse},
};
use serde::Serialize;

use crate::{
    configuration::{Configuration, NodeType, ValidatorAddress},
    encryption::generate_rsa_key_pair,
    model::{HexString, BlockChain, PublicKeyStr},
};
use anyhow::{anyhow, Result};

pub trait RequestHandler<T: Serialize> {
    type RESPONSE;
    fn handle_request(&self, blockchain: &mut BlockChain, configuration: &mut Configuration) -> Result<Self::RESPONSE>;
}

impl RequestHandler<InternalResponse> for InternalRequest {
    type RESPONSE = InternalResponse;
    fn handle_request(&self, blockchain: &mut BlockChain, configuration: &mut Configuration) -> Result<Self::RESPONSE> {
        match &self.command {
            protocol::internal::CommandRequest::OnBoardValidator { return_address } => {
                match configuration.node_type {
                    NodeType::Coordinator { ref mut validators } => {
                        let validator_address = ValidatorAddress(return_address.to_owned());
                        validators.push(validator_address);
                        println!(
                            "Added new validator {:?}, total validators {}",
                            return_address,
                            &validators.len()
                        );
                        Ok(())
                    }
                    NodeType::Validator => {
                        Err(anyhow!("Validator can't on-board another validator"))
                    }
                }?;

                Ok(InternalResponse::Success {
                    request_id: self.request_id.to_owned(),
                    response: CommandResponse::OnBoardValidatorResponse,
                })
            }
            protocol::internal::CommandRequest::ValidateAndCommitTransaction {
                from,
                to,
                amount,
                signature,
            } => {
                if configuration.node_type != NodeType::Validator {
                    return Err(anyhow!("Node has to be a validator"));
                }

                Ok(InternalResponse::Success {
                    request_id: self.request_id.to_owned(),
                    response: CommandResponse::OnBoardValidatorResponse,
                })
            }
        }
    }
}

impl RequestHandler<ExternalResponse> for ExternalRequest {
    type RESPONSE = ExternalResponse;
    fn handle_request(&self, blockchain: &mut BlockChain, configuration: &mut Configuration) -> Result<Self::RESPONSE> {
        println!("External request received");
        match &self.command {
            UserCommand::CreateRecord { data } => panic!("Not ready yet"),
            UserCommand::PingCommand { msg } => {
                println!("Received ping command");
                Ok(ExternalResponse::Success(
                    UserCommandResponse::PingCommandResponse {
                        request_id: self.request_id.clone(),
                        msg: format!("Original message: {}, PONG PONG", msg),
                    },
                ))
            }
            UserCommand::GenerateWallet => {
                let (priv_k, pub_k) = &generate_rsa_key_pair()?;
                Ok(ExternalResponse::Success(UserCommandResponse::GenerateWalletResponse {
                    private_key: HexString::try_from(priv_k)?.0,
                    public_key: HexString::try_from(pub_k)?.0,
                }))
            }
            UserCommand::GenerateNonce { address } => {
                let nonce = blockchain.request_nonce_for_address(&PublicKeyStr::from_str(&address[..]));

                Ok(ExternalResponse::Success(UserCommandResponse::GenerateNonceResponse {
                    nonce
                }))
            },
        }
    }
}
