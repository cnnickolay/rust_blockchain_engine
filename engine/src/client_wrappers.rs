use anyhow::{Result, anyhow};
use mockall::automock;
use protocol::{request::{CommandRequest, CommandResponse}, common::Validator};

use crate::{client::{Client, send_bytes}, runtime::configuration::ValidatorAddress};

struct ClientWrapperImpl;

#[automock]
pub trait ClientWrapper {
    fn send_blockchain_tip_request(&self, address: &ValidatorAddress, sender: &Validator) -> Result<String>;
}

impl ClientWrapper for ClientWrapperImpl {
    fn send_blockchain_tip_request(&self, address: &ValidatorAddress, sender: &Validator) -> Result<String> {
        let request = CommandRequest::BlockchainTip.to_request(&sender);
        let response = send_bytes(&address.0, &request)?;
        
        match response.body {
            protocol::request::ResponseBody::Success(CommandResponse::BlockchainTipResponse { blockchain_tip_hash }) => 
                Ok(blockchain_tip_hash.to_owned()),
            protocol::request::ResponseBody::Success(unexpected_response) => 
                Err(anyhow!(format!("Unexpected response for {}: {}", request.command.name(), unexpected_response.name())),),
            protocol::request::ResponseBody::Error { msg } => Err(anyhow!(msg)),
        }
    }
}