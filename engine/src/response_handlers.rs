use std::{sync::{Arc, Mutex}, borrow::BorrowMut};

use anyhow::Result;
use protocol::{request::CommandResponse, request::{Request, Response, CommandRequest, ValidatorWithSignature, Validator}};

use crate::{blockchain::blockchain::BlockChain, model::{PublicKeyStr, Signature}, configuration::{ValidatorReference, ValidatorAddress, Configuration}};

use super::blockchain::validator_signature::ValidatorSignature;

pub fn handle_response(blockchain: &Arc<Mutex<BlockChain>>, configuration: &mut Configuration, request_id: &str, response: &Response) -> Result<Vec<(ValidatorReference, Request)>> {
    match response {
        Response::Success { request_id, response } => handle_command(blockchain.lock().unwrap().borrow_mut(), configuration, request_id, response),
        Response::Error { msg } => Err(anyhow::anyhow!(msg.to_owned())),
    }
}

fn handle_command(blockchain: &mut BlockChain, configuration: &mut Configuration, request_id: &str, response: &CommandResponse) -> Result<Vec<(ValidatorReference, Request)>> {
    match response {
        CommandResponse::OnBoardValidatorResponse { validators } => {
            let new_validators: Vec<_> = validators.iter().map(|v| (PublicKeyStr::from_str(&v.public_key), ValidatorAddress(v.address.to_owned()))).collect();
            configuration.add_validators(&new_validators);
            println!("Validators added: {:?}", configuration.validators.iter().map(|validator| &validator.1).collect::<Vec<&ValidatorAddress>>());

            ok()
        },
        CommandResponse::SynchronizeBlockchainResponse {} => ok(),
        CommandResponse::RequestTransactionValidationResponse { 
            old_blockchain_tip, new_blockchain_tip, 
            validator_public_key, 
            transaction_cbor, 
            validator_signature: _validator_signature
        } => {
            let last = blockchain.blocks.last_mut().unwrap();
            let validator_signature = ValidatorSignature::new(&PublicKeyStr::from_str(&validator_public_key), &Signature::from_string(&_validator_signature));
            let validator_signature_json = serde_json::to_string_pretty(&validator_signature)?;
            last.validator_signatures.push(validator_signature);
            println!("New validation added (total {}) {}", last.validator_signatures.len(), validator_signature_json);

            let prev_block = "not needed atm"; // &blockchain.blocks[blockchain.blocks.len() - 2];
            let current_block = &blockchain.blocks[blockchain.blocks.len() - 1];

            let requests = configuration.validators.iter().flat_map(|(validator_pub_key, validator_addr)| {
                if *validator_pub_key != configuration.validator_public_key {
                    if let Some(validator_address) = configuration.find_validator_address_by_key(validator_pub_key) {
                        let command = CommandRequest::SynchronizeBlockchain {
                            signatures: vec![ValidatorWithSignature { 
                                validator: Validator { address: validator_address.0.to_owned(), public_key: validator_public_key.to_owned() }, signature: _validator_signature.to_owned() 
                            }],
                            transaction_cbor: transaction_cbor.to_owned(),
                            blockchain_tip_before_transaction: prev_block.to_owned(),
                            blockchain_tip_after_transaction: current_block.hash.to_owned(),
                        };
                        println!("Synchronisation request will be sent to {}", validator_address.0);
                        vec![((validator_pub_key.clone(), validator_address), Request { request_id: request_id.to_owned(), command })]
                    } else {
                        println!("Validator {} is not registered with this node", validator_pub_key);
                        Vec::new()
                    }
                } else {
                    Vec::new()
                }
            }).collect();

            ok_with_requests(requests)
        },
        CommandResponse::PingCommandResponse { msg } => todo!(),
        CommandResponse::GenerateWalletResponse { private_key, public_key } => todo!(),
        CommandResponse::PrintBalancesResponse { balances } => todo!(),
        CommandResponse::BalanceTransactionResponse { request_id, body, cbor } => todo!(),
        CommandResponse::CommitTransactionResponse { blockchain_hash } => todo!(),
        CommandResponse::PrintBlockchainResponse { blocks } => todo!(),
    }
}

fn ok() -> Result<Vec<(ValidatorReference, Request)>> {
    Ok(Vec::new())
}

fn ok_with_requests(requests: Vec<(ValidatorReference, Request)>) -> Result<Vec<(ValidatorReference, Request)>> {
    Ok(requests)
}
