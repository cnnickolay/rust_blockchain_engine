use std::{net::TcpStream, io::Write};

use anyhow::{Result, anyhow};
use log::trace;
use protocol::{request::Request, request::{CommandResponse, Response, CommandRequest, ResponseBody}};
use rsa::RsaPrivateKey;

use crate::{model::PrivateKeyStr, blockchain::{cbor::Cbor, balanced_transaction::BalancedTransaction}};

pub struct Client {
    destination: String,
}

impl Client {
    pub fn new(destination: &str) -> Self {
        Client { destination: destination.to_string() }
    }

    pub fn ping(&self, msg: &str) -> Result<Response> {
        send_bytes(&self.destination, CommandRequest::new_ping(msg).to_client_request())
    }

    pub fn generate_wallet(&self) -> Result<Response> {
        send_bytes(&self.destination, CommandRequest::GenerateWallet.to_client_request())
    }

    pub fn print_balances(&self) -> Result<Response> {
        send_bytes(&self.destination, CommandRequest::PrintBalances.to_client_request())
    }

    pub fn print_validators(&self) -> Result<String> {
        let response = send_bytes(&self.destination, CommandRequest::PrintValidators.to_client_request())?;
        if let Response {body: ResponseBody::Success (CommandResponse::PrintValidatorsResponse(response)), ..} = response {
            let validators: Vec<String> = response.validators.iter().map(|v| format!("{} #### {}", v.address, &v.public_key[0..40])).collect();
            Ok(validators.join("\n"))
        } else {
            Err(anyhow!("Unexpected response for print_blockchain: {:?}", response))
        }
    }

    pub fn balance_transaction(&self, from: &str, to: &str, amount: u64) -> Result<Response> {
        send_bytes(&self.destination, CommandRequest::new_balance_transaction(from, to, amount).to_client_request())
    }

    pub fn commit_transaction(&self, cbor: &str, private_key: &str) -> Result<Response> {
        let rsa_private_key = RsaPrivateKey::try_from(&PrivateKeyStr(private_key.to_string()))?;
        let balanced_transaction = BalancedTransaction::try_from(&Cbor::new(cbor))?;

        let signed_transaction = balanced_transaction.sign(&rsa_private_key)?;
        let signed_cbor: Cbor = (&signed_transaction).try_into()?;
    
        send_bytes(&self.destination, CommandRequest::new_commit_transaction(&signed_cbor.0).to_client_request())
    }

    pub fn print_blockchain(&self) -> Result<String> {
        let response = send_bytes(&self.destination, CommandRequest::PrintBlockchain.to_client_request())?;
        if let Response {body: ResponseBody::Success (CommandResponse::PrintBlockchainResponse{blocks}), ..} = response {
            Ok(blocks.join("\n\n"))
        } else {
            Err(anyhow!("Unexpected response for print_blockchain: {:?}", response))
        }
    }
}

pub fn send_bytes(destination: &str, msg: Request) -> Result<Response> {
    trace!("Sending {:?}", msg);
    let mut stream = TcpStream::connect(destination)?;

    let bytes = serde_cbor::to_vec(&msg)?;
    let len: [u8; 8] = bytes.len().to_be_bytes();
    stream.write(&len)?;
    stream.write(&bytes)?;

    let response: Response = serde_cbor::from_reader(&stream)?;

    Ok(response)
}

