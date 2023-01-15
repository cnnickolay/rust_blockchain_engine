use std::{sync::{Mutex, Arc}};

use protocol::{
    request::{CommandResponse, CommandRequest, Validator, ValidatorWithSignature, self, Response}, request::{Request, ResponseBody},
};

use crate::{
    configuration::{Configuration, ValidatorAddress, ValidatorReference},
    encryption::generate_rsa_key_pair,
    model::{HexString, PublicKeyStr, Signature}, blockchain::{blockchain::BlockChain, transaction::Transaction, signed_balanced_transaction::{SignedBalancedTransaction}, cbor::Cbor, validator_signature::ValidatorSignature}, utils::shorten_long_string,
};
use anyhow::{Result, anyhow};

pub fn handle_request(
    request: &Request,
    blockchain: Arc<Mutex<BlockChain>>,
    configuration: &mut Configuration,
) -> Result<(Response, Vec<(ValidatorReference, Request)>)> {
    match &request.command {
        protocol::request::CommandRequest::OnBoardValidator { return_address: new_validator_address, public_key: new_validator_public_key } => {
            let blockchain = blockchain.lock().unwrap();
            let mut requests = Vec::new();

            for validator in &configuration.validators {
                let request = request::CommandRequest::new_on_board_command(&new_validator_address, &new_validator_public_key).to_request_with_id(configuration.validator(), &request.request_id);
                requests.push((validator.clone(), request));
            }

            let validator_address = ValidatorAddress(new_validator_address.to_owned());
            configuration.add_validators(&[ValidatorReference { pk: PublicKeyStr::from_str(&new_validator_public_key), address: validator_address } ]);
            println!(
                "Added new validator {:?}, total validators {}",
                new_validator_address,
                &configuration.validators.len()
            );

            let mut all_validators: Vec<Validator> = 
                configuration.validators.iter()
                .map(|ValidatorReference { pk: public_key, address: addr } | {
                    Validator { public_key: public_key.0.0.to_owned(), address: addr.0.clone()}
                })
                .collect();
            all_validators.push(Validator { 
                address: format!("{}:{}", configuration.ip, configuration.port), 
                public_key: configuration.validator_public_key.0.0.to_owned()
            });

            let response = Response {
                orig_request_id: request.request_id.to_owned(),
                replier: configuration.validator(),
                body: ResponseBody::Success(CommandResponse::OnBoardValidatorResponse { 
                    on_boarding_validator: Validator { address: configuration.address(), public_key: String::from(&configuration.validator_public_key) },
                    validators: all_validators, 
                    blockchain_tip: blockchain.blockchain_hash()?
                }),
            };
            ok_with_requests(response, requests)
        },
        CommandRequest::PingCommand { msg } => {
            println!("Received ping command");
            success(&request.request_id, configuration.validator(), CommandResponse::PingCommandResponse {
                msg: format!("Original message: {}, PONG PONG", msg),
            })
        },
        CommandRequest::GenerateWallet => {
            let (priv_k, pub_k) = &generate_rsa_key_pair()?;
            success(&request.request_id, configuration.validator(), CommandResponse::GenerateWalletResponse {
                private_key: HexString::try_from(priv_k)?.0,
                public_key: HexString::try_from(pub_k)?.0,
            })
        },
        CommandRequest::PrintBalances => {
            let balances =
                    blockchain.lock().unwrap()
                        .all_balances()
                        .iter()
                        .map(|(k, v)| (shorten_long_string(&k.0 .0), v.clone()))
                        .collect();

            success(&request.request_id, configuration.validator(), CommandResponse::PrintBalancesResponse { balances })
        },
        
        CommandRequest::BalanceTransaction { from, to, amount } => {
            let blockchain = blockchain.lock().unwrap();
            let balanced_transaction = &Transaction::new(&PublicKeyStr::from_str(from), &PublicKeyStr::from_str(to), *amount)
                .balance_transaction(&blockchain)?;
            let cbor_bytes = balanced_transaction.to_cbor()?;
            let cbor = hex::encode(&cbor_bytes);
            let body = serde_json::to_string_pretty(balanced_transaction)?;

            success(&request.request_id, configuration.validator(), CommandResponse::BalanceTransactionResponse { request_id: request.request_id.clone(), body, cbor })
        },

        CommandRequest::CommitTransaction { signed_transaction_cbor } => {
            let mut blockchain = blockchain.lock().unwrap();
            let blockchain_previous_tip = blockchain.blockchain_hash()?;
            let signed_transaction = SignedBalancedTransaction::try_from(&Cbor::new(&signed_transaction_cbor))?;
            let block = signed_transaction.commit(&mut blockchain, &configuration.validator_private_key)?;
            let validator_signature = block.validator_signatures.first().unwrap();

            let mut requests = Vec::new();

            for validator in &configuration.validators {
                let request = CommandRequest::RequestTransactionValidation {
                    blockchain_previous_tip: blockchain_previous_tip.to_owned(),
                    blockchain_new_tip: block.hash.to_owned(),
                    transaction_cbor: signed_transaction_cbor.to_owned(),
                    validator_signature: ValidatorWithSignature {
                        validator: Validator {
                            address: configuration.address(), 
                            public_key: configuration.validator_public_key.0.0.to_owned()
                        },
                        signature: validator_signature.validator_signature.0.0.to_owned()
                    },
                    validator: Validator { address: configuration.address(), public_key: configuration.validator_public_key.0.0.to_owned() },
                }.to_request(&configuration.validator());

                requests.push((validator.clone(), request));
            }

            println!("{}", serde_json::to_string_pretty(&block)?);

            let response = Response {
                orig_request_id: request.request_id.to_owned(),
                replier: configuration.validator(),
                body: ResponseBody::Success (
                    CommandResponse::CommitTransactionResponse { blockchain_hash: block.hash.to_owned() },
                ),
            };
            ok_with_requests(response, requests)
        },
        CommandRequest::RequestTransactionValidation { blockchain_previous_tip, blockchain_new_tip, transaction_cbor, validator_signature: sender_validator_signature, validator } => {
            let mut blockchain = blockchain.lock().unwrap();
            let blockchain_hash = blockchain.blockchain_hash()?;
            if *blockchain_previous_tip != blockchain_hash {
                let msg = format!("Transaction can't be applied for blockchains are not in sync: {} != {}", blockchain_previous_tip, blockchain_hash);
                println!("{}", msg);
                return err(&request.request_id, configuration.validator(), &msg);
            }

            let signed_transaction = SignedBalancedTransaction::try_from(&Cbor::new(&transaction_cbor))?;
            let block = signed_transaction.commit(&mut blockchain, &configuration.validator_private_key)?;
            let blockchain_hash = blockchain.blockchain_hash()?;
            let validator_signature = block.validator_signatures.first().ok_or(anyhow!("Transaction wasn't signed by validator"))?;

            if *blockchain_new_tip != block.hash {
                let msg = format!("Blockchain hash is different. Possibility of a hard fork");
                println!("{}", msg);
                return err(&request.request_id, configuration.validator(), &msg);
            }

            println!("Transaction successfully verified and added to blockchain. Total verifications: {}", block.validator_signatures.len());
            println!("{}", serde_json::to_string_pretty(&block)?);

            // Add signature from the sender validator into the block
            let last = blockchain.blocks.last_mut().unwrap();
            last.validator_signatures.push(
                ValidatorSignature::new(
                    &PublicKeyStr::from_str(&validator.public_key), 
                    &Signature::from_string(&sender_validator_signature.signature)
                )
            );

            success(&request.request_id, configuration.validator(), 
                CommandResponse::RequestTransactionValidationResponse {
                    new_blockchain_tip: blockchain_hash,
                    validator_public_key: configuration.validator_public_key.0.0.to_owned(),
                    transaction_cbor: transaction_cbor.to_owned(),
                    validator_signature: validator_signature.validator_signature.0.0.to_owned(),
                    old_blockchain_tip: blockchain_previous_tip.to_owned(),
                },
            )
        },
        CommandRequest::SynchronizeBlockchain { signatures, transaction_cbor, blockchain_tip_before_transaction, blockchain_tip_after_transaction  } => {
            println!("Synchronization request received");
            let mut blockchain = blockchain.lock().unwrap();
            let last = blockchain.blocks.last_mut().unwrap();

            if last.hash != *blockchain_tip_after_transaction {
                return err(&request.request_id, configuration.validator(), &format!("Blockchain tips are different, synchronization needed. Incoming tip: {}, this blockchain tip: {}", blockchain_tip_after_transaction, last.hash));
            }

            if signatures.len() > 1 {
                return err(&request.request_id, configuration.validator(), &format!("Only one signature is supported by SynchronizeBlockchain for now, received {}", signatures.len()));
            }

            let signature = &signatures[0];
            last.validator_signatures.push(signature.into());

            success(&request.request_id, configuration.validator(), CommandResponse::SynchronizeBlockchainResponse{})
        },
        CommandRequest::PrintBlockchain => {
            let blockchain = blockchain.lock().unwrap();
            let blocks = blockchain.blocks.iter().enumerate().map(|(idx, block)| {
                let mut block_str = String::new();
                block_str.push_str(&format!("{}. Block {}", idx + 1, block.hash));
                block_str.push_str("\n  Input UTxOs:");
                for (idx, input_utxo) in block.transaction.inputs().iter().enumerate() {
                    block_str.push_str(&format!("\n    Input {}:", idx + 1));
                    block_str.push_str(&format!("\n      Addr: {}", shorten_long_string(&input_utxo.address.0.0)));
                    block_str.push_str(&format!("\n      Amount: {}", input_utxo.amount));
                }
                block_str.push_str("\n  Output UTxOs:");
                for (idx, output_utxo) in block.transaction.outputs().iter().enumerate() {
                    block_str.push_str(&format!("\n    Output {}:", idx + 1));
                    block_str.push_str(&format!("\n      Addr: {}", shorten_long_string(&output_utxo.address.0.0)));
                    block_str.push_str(&format!("\n      Amount: {}", output_utxo.amount));
                }
                block_str.push_str(&format!("\n  Transaction signature: {}", shorten_long_string(&block.transaction.signature.0.0)));
                block_str.push_str(&format!("\n  Confirmations (total {}):", block.validator_signatures.len()));
                for (idx, signature) in block.validator_signatures.iter().enumerate() {
                    block_str.push_str(&format!("\n    Confirmation {}:", idx + 1));
                    block_str.push_str(&format!("\n      Validator Id: {}", shorten_long_string(&signature.validator_public_key.0.0)));
                    block_str.push_str(&format!("\n      Signature: {}", shorten_long_string(&signature.validator_signature.0.0)));
                }

                block_str
            }).collect();
            success(&request.request_id, configuration.validator(), CommandResponse::PrintBlockchainResponse { blocks })
        },

        CommandRequest::RequestSynchronization { blockchain_tip } => {
            println!("Request synchronization received for tip {}", blockchain_tip);
            let blockchain = blockchain.lock().unwrap();
            let block_index = blockchain.index_of_block(blockchain_tip);
            println!("Found block at {}", block_index);

            if blockchain.blockchain_hash()? == *blockchain_tip {
                return err(&request.request_id, configuration.validator(), "Fully synchronized");
            }

            if block_index >= 0 || blockchain.initial_utxo.hash_str() == *blockchain_tip {
                let next_block = &blockchain.blocks[(block_index + 1) as usize];
                let next_hash = next_block.hash.to_owned();
                let previous_hash = if block_index == -1 {
                    blockchain.initial_utxo.hash_str()
                } else {
                    let hash = &blockchain.blocks[block_index as usize].hash;
                    hash.to_owned()
                };

                let response = CommandResponse::RequestSynchronizationResponse {
                    previous_hash, 
                    next_hash, 
                    transaction_cbor: Cbor::try_from(&next_block.transaction)?.0, 
                    signatures: next_block.validator_signatures.iter().map(|v| ValidatorWithSignature::from(v)).collect() 
                };

                return success(&request.request_id, configuration.validator(), response);
            } else {
                let err_msg = format!("Impossible to synchronize, no common ancestor for hash {}", blockchain_tip);
                return err(&request.request_id, configuration.validator(), &err_msg);
            }
        },
    }
}

fn ok(response: Response) -> Result<(Response, Vec<(ValidatorReference, Request)>)> {
    Ok((response, Vec::new()))
}

fn success(request_id: &str, validator: Validator, command_response: CommandResponse) -> Result<(Response, Vec<(ValidatorReference, Request)>)> {
    let body = ResponseBody::Success ( command_response );
    let response = Response {
        orig_request_id: request_id.to_owned(),
        replier: validator,
        body,
    };
    ok(response)

} 

fn err(request_id: &str, validator: Validator, msg: &str) -> Result<(Response, Vec<(ValidatorReference, Request)>)> {
    let body = ResponseBody::Error { msg: msg.to_owned() };
    let response = Response {
        orig_request_id: request_id.to_owned(),
        replier: validator,
        body,
    };
    ok(response)
}

fn ok_with_requests(response: Response, requests: Vec<(ValidatorReference, Request)>) -> Result<(Response, Vec<(ValidatorReference, Request)>)> {
    Ok((response, requests))
}
