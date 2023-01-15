use std::{sync::{Arc, Mutex}, borrow::BorrowMut};

use anyhow::{Result, anyhow};
use protocol::{request::CommandResponse, request::{Request, Response, CommandRequest, ValidatorWithSignature, Validator, ResponseBody}};

use crate::{blockchain::{blockchain::BlockChain, signed_balanced_transaction::SignedBalancedTransaction, cbor::Cbor}, model::{PublicKeyStr, Signature}, configuration::{ValidatorReference, ValidatorAddress, Configuration}};

use super::blockchain::validator_signature::ValidatorSignature;

pub fn handle_response(blockchain: &Arc<Mutex<BlockChain>>, configuration: &mut Configuration, request_id: &str, response: &Response) -> Result<Vec<(ValidatorReference, Request)>> {
    match response {
        Response { orig_request_id, replier, body: ResponseBody::Success(response) } => handle_command(blockchain.lock().unwrap().borrow_mut(), configuration, request_id, &replier, response),
        Response { orig_request_id, replier, body: ResponseBody::Error { msg } } => Err(anyhow!(msg.to_owned())),
    }
}

fn handle_command(blockchain: &mut BlockChain, configuration: &mut Configuration, request_id: &str, replier: &Validator, response: &CommandResponse) -> Result<Vec<(ValidatorReference, Request)>> {
    match response {
        CommandResponse::OnBoardValidatorResponse { on_boarding_validator, validators, blockchain_tip } => {
            let new_validators: Vec<_> = validators.iter().map(|v| ValidatorReference { pk: PublicKeyStr::from_str(&v.public_key), address: ValidatorAddress(v.address.to_owned())}).collect();
            configuration.add_validators(&new_validators);
            println!("Validators added: {:?}", configuration.validators.iter().map(|validator| &validator.address).collect::<Vec<&ValidatorAddress>>());

            let this_blockchain_tip = blockchain.blockchain_hash()?;
            if this_blockchain_tip != *blockchain_tip {
                let validator = ValidatorReference::from(on_boarding_validator);
                let command = CommandRequest::RequestSynchronization {
                    blockchain_tip: this_blockchain_tip,
                };
                return ok_with_requests(vec![(on_boarding_validator.into(), command.to_request(&configuration.validator()))]);
            }

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

            let requests = configuration.validators.iter().flat_map(|ValidatorReference { pk: validator_pub_key, address: validator_addr } | {
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
                        let request = command.to_request(&configuration.validator());
                        vec![(ValidatorReference { pk: validator_pub_key.clone(), address: validator_address }, request)]
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
        CommandResponse::RequestSynchronizationResponse { previous_hash, next_hash, transaction_cbor, signatures } => {
            println!("Processing RequestSynchronizationResponse. Base hash {}, expected hash {}", previous_hash, next_hash);
            let current_blockchain_tip = blockchain.blockchain_hash()?;
            if current_blockchain_tip != *previous_hash {
                println!("RequestSynchronizationResponse is impossible because base hash from the requester {} does not match to base hash of the receiver {}", previous_hash, current_blockchain_tip);
                return ok();
            }

            let signed_transaction = SignedBalancedTransaction::try_from(&Cbor::new(&transaction_cbor))?;
            let block = signed_transaction.commit(blockchain, &configuration.validator_private_key)?;
            let validator_signature = block.validator_signatures.first().ok_or(anyhow!("Transaction wasn't signed by validator"))?;

            println!("Transaction applied, new block hash is {}", block.hash);

            if block.hash.to_owned() != *next_hash {
                println!("RequestSynchronizationResponse failed because resulting hash {} does not match to expected hash {}", block.hash, next_hash);
                return ok();
            }

            let command = CommandRequest::RequestSynchronization {
                blockchain_tip: block.hash.to_owned(),
            };
            let request = command.to_request(&configuration.validator());

            ok_with_requests(vec![(ValidatorReference::from(replier), request)])
        },
    }
}

fn ok() -> Result<Vec<(ValidatorReference, Request)>> {
    Ok(Vec::new())
}

fn ok_with_requests(requests: Vec<(ValidatorReference, Request)>) -> Result<Vec<(ValidatorReference, Request)>> {
    Ok(requests)
}
