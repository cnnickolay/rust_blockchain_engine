use std::{net::TcpStream, io::Write, thread, sync::{Arc, Mutex}};

use anyhow::{Result, anyhow};
use protocol::{request::Request, response::Response, external::{UserCommand}, internal::{self, InternalResponse, CommandResponse, Validator, CommandRequest, ValidatorSignature}};
use rsa::RsaPrivateKey;

use crate::{model::{PublicKeyStr, PrivateKeyStr, Signature}, blockchain::{cbor::Cbor, balanced_transaction::BalancedTransaction, blockchain::BlockChain, block}};

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

    pub fn register_validator(&self, address: &str, public_key: &PublicKeyStr, retransmitted: bool) -> Result<Vec<Validator>> {
        let response = send_bytes(&self.destination, internal::CommandRequest::new_on_board_command(&address, &public_key.0.0, retransmitted).to_request())?;
        match response {
            Response::Internal(InternalResponse::Success {response: CommandResponse::OnBoardValidatorResponse{validators}, ..}) => Ok(validators),
            bad_response => Err(anyhow!("Wrong response for registering validator: {:?}", bad_response))
        } 
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

    pub fn request_transaction_validation(&self, 
                                 blockchain: Arc<Mutex<BlockChain>>,
                                 blockchain_previous_tip: &str, 
                                 blockchain_new_tip: &str, 
                                 transaction_cbor: &str, 
                                 validator_signature: &ValidatorSignature) -> () {
        let command = CommandRequest::RequestTransactionValidation {
            blockchain_previous_tip: blockchain_previous_tip.to_owned(),
            blockchain_new_tip: blockchain_new_tip.to_owned(),
            transaction_cbor: transaction_cbor.to_owned(),
            validator_signature: validator_signature.clone(),
        };
        let destination = self.destination.to_owned();

        thread::spawn(move || {
            let err_msg = format!("Error happened when sending {}", serde_json::to_string_pretty(&command).unwrap());
            let response = send_bytes(&destination, command.to_request());
            match response {
                Ok(Response::Internal(InternalResponse::Success { response: CommandResponse::RequestTransactionValidationResponse{validator_public_key, validator_signature, ..}, .. })) => {
                    if let Ok(mut blockchain) = blockchain.lock() {
                        let last = blockchain.blocks.last_mut().unwrap();
                        last.validator_signatures.push(super::blockchain::validator_signature::ValidatorSignature::new(&PublicKeyStr::from_str(&validator_public_key), &Signature::from_string(&validator_signature)));
                        println!("Validation added {}", serde_json::to_string_pretty(&last.validator_signatures).unwrap());
                    }
                }
                Ok(Response::Internal(InternalResponse::Error {msg})) => println!("{}", err_msg),
                _ => {}
            }
        });
    }

    pub fn print_blockchain(&self) -> Result<Response> {
        send_bytes(&self.destination, UserCommand::PrintBlockchain.to_request())
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

