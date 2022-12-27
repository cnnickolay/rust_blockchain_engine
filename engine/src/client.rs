use std::{net::TcpStream, io::Write};

use anyhow::{Result, anyhow};
use protocol::{request::Request, response::Response, external::{UserCommand, ExternalResponse, UserCommandResponse}, internal};
use rsa::RsaPrivateKey;

use crate::model::{Transaction, PublicKeyStr, PrivateKeyStr};

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

    pub fn generate_nonce(&self, address: &str) -> Result<Response> {
        send_bytes(&self.destination, UserCommand::new_generate_nonce(address).to_request())
    }

    pub fn print_balances(&self) -> Result<Response> {
        send_bytes(&self.destination, UserCommand::PrintBalances.to_request())
    }

    pub fn send_transaction(&self, from: &str, to: &str, amount: u64) -> Result<Response> {
        let rsa_private_key = RsaPrivateKey::try_from(&PrivateKeyStr(from.to_string()))?;
        let public_key = PublicKeyStr::try_from(&rsa_private_key.to_public_key())?;

        let nonce = self.generate_nonce(&public_key.0.0)?;

        if let Response::External(ExternalResponse::Success(UserCommandResponse::GenerateNonceResponse{ref nonce})) = nonce {
            let signed_transaction = Transaction::new_unsigned(nonce.to_owned(), public_key.clone(), PublicKeyStr::from_str(to), amount)
                .sign(&rsa_private_key)?;
        
            send_bytes(&self.destination, UserCommand::new_transaction(nonce, &public_key.0.0, to, amount, &signed_transaction.signature.0.0).to_request())
        } else {
            Err(anyhow!("Failed to retrieve nonce"))
        }
    }
}

pub fn send_bytes(destination: &str, msg: Request) -> Result<Response> {
    println!("Sending {:?}", msg);
    let mut stream = TcpStream::connect(destination)?;

    let bytes = serde_cbor::to_vec(&msg)?;
    let len: [u8; 8] = bytes.len().to_be_bytes();
    stream.write(&len)?;
    stream.write(&bytes)?;

    let response: Response = serde_cbor::from_reader(&stream)?;

    Ok(response)
}

