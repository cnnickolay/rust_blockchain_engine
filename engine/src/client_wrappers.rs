use std::{net::TcpStream, io::Write};

use anyhow::{anyhow, Result};
use log::trace;
use mockall::automock;
use protocol::common::Validator;

use crate::{runtime::configuration::ValidatorAddress, model::requests::{Request, Response, ResponseBody, CommandResponse, CommandRequest, BlockchainTipRequest}};

pub struct ClientWrapperImpl;

#[automock]
pub trait ClientWrapper {
    fn send_blockchain_tip_request(
        &self,
        address: &ValidatorAddress,
        sender: &Validator,
    ) -> Result<String>;
}

impl ClientWrapper for ClientWrapperImpl {
    fn send_blockchain_tip_request(
        &self,
        address: &ValidatorAddress,
        sender: &Validator,
    ) -> Result<String> {
        let request = CommandRequest::BlockchainTip(BlockchainTipRequest).to_request(&sender);
        let response = send_bytes(&address.0, &request)?;

        match response.body {
            ResponseBody::Success(CommandResponse::BlockchainTip(response)) => Ok(response.blockchain_tip_hash.to_owned()),
            ResponseBody::Success(unexpected_response) => Err(anyhow!(format!(
                "Unexpected response for {}: {}",
                request.command.name(),
                unexpected_response.name()
            ))),
            ResponseBody::Error { msg } => Err(anyhow!(msg)),
        }
    }
}

pub fn send(validator: &ValidatorAddress, msg: &Request) -> Result<Response> {
    send_bytes(&validator.0, msg)
}

pub fn send_bytes(destination: &str, msg: &Request) -> Result<Response> {
    trace!("Sending {:?}", msg);
    let mut stream = TcpStream::connect(destination)?;

    let bytes = serde_cbor::to_vec(&msg)?;
    let len: [u8; 8] = bytes.len().to_be_bytes();
    stream.write(&len)?;
    stream.write(&bytes)?;

    let response: Response = serde_cbor::from_reader(&stream)?;

    Ok(response)
}
