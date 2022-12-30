use std::{net::TcpStream, io::Write};

use anyhow::{Result, anyhow};
use protocol::{request::Request, response::Response, external::{UserCommand, ExternalResponse, UserCommandResponse}, internal};
use rsa::RsaPrivateKey;

use crate::{model::{PublicKeyStr, PrivateKeyStr}, blockchain::{transaction::Transaction, cbor::Cbor, balanced_transaction::BalancedTransaction}};

pub struct Client {
    destination: String,
}

impl Client {
    pub fn new(destination: &str) -> Self {
        Client { destination: destination.to_string() }
    }

    pub fn ping(&self, msg: &str) -> Result<Response> {
        send_bytes(&self.destination, UserCommand::new_ping(msg).to_request())
    }

    pub fn register_validator(&self, ip: &str, port: u16) -> Result<Response> {
        send_bytes(&self.destination, internal::CommandRequest::new_on_board_command(&format!("{}:{}", ip, port)).to_request())
    }

    pub fn generate_wallet(&self) -> Result<Response> {
        send_bytes(&self.destination, UserCommand::GenerateWallet.to_request())
    }

    pub fn print_balances(&self) -> Result<Response> {
        send_bytes(&self.destination, UserCommand::PrintBalances.to_request())
    }

    pub fn balance_transaction(&self, from: &str, to: &str, amount: u64) -> Result<Response> {
        send_bytes(&self.destination, UserCommand::new_balance_transaction(from, to, amount).to_request())
    }

    pub fn commit_transaction(&self, cbor: &str, private_key: &str) -> Result<Response> {
        let rsa_private_key = RsaPrivateKey::try_from(&PrivateKeyStr(private_key.to_string()))?;
        let balanced_transaction = BalancedTransaction::try_from(&Cbor::new(cbor))?;

        let signed_transaction = balanced_transaction.sign(&rsa_private_key)?;
        let signed_cbor: Cbor = (&signed_transaction).try_into()?;
    
        send_bytes(&self.destination, UserCommand::new_commit_transaction(&signed_cbor.0).to_request())
    }
}

pub fn send_bytes(destination: &str, msg: Request) -> Result<Response> {
    // println!("Sending {:?}", msg);
    let mut stream = TcpStream::connect(destination)?;

    let bytes = serde_cbor::to_vec(&msg)?;
    let len: [u8; 8] = bytes.len().to_be_bytes();
    stream.write(&len)?;
    stream.write(&bytes)?;

    let response: Response = serde_cbor::from_reader(&stream)?;

    Ok(response)
}

