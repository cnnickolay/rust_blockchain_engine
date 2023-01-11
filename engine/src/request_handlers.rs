use std::{sync::{Mutex, Arc}};

use protocol::{
    request::{CommandResponse, CommandRequest, Validator, ValidatorWithSignature, self, Response}, request::Request,
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

            let mut requests = Vec::new();

            for validator in &configuration.validators {
                let request = request::CommandRequest::new_on_board_command(&new_validator_address, &new_validator_public_key).to_request_with_id(&request.request_id);
                requests.push((validator.clone(), request));
            }

            let validator_address = ValidatorAddress(new_validator_address.to_owned());
            configuration.add_validators(&[(PublicKeyStr::from_str(&new_validator_public_key), validator_address)]);
            println!(
                "Added new validator {:?}, total validators {}",
                new_validator_address,
                &configuration.validators.len()
            );

            let mut all_validators: Vec<Validator> = 
                configuration.validators.iter()
                .map(|(public_key, addr)| {
                    Validator { public_key: public_key.0.0.to_owned(), address: addr.0.clone()}
                })
                .collect();
            all_validators.push(Validator { 
                address: format!("{}:{}", configuration.ip, configuration.port), 
                public_key: configuration.validator_public_key.0.0.to_owned()
            });

            ok_with_requests(Response::Success {
                request_id: request.request_id.to_owned(),
                response: CommandResponse::OnBoardValidatorResponse { validators: all_validators },
            }, requests)
        },
        CommandRequest::SynchronizeBlockchain { address, blockchain_hash } => todo!(),
        CommandRequest::PingCommand { msg } => {
            println!("Received ping command");
            ok(Response::Success {
                request_id: request.request_id.clone(),
                response: CommandResponse::PingCommandResponse {
                    msg: format!("Original message: {}, PONG PONG", msg),
                },
            })
        },
        CommandRequest::GenerateWallet => {
            let (priv_k, pub_k) = &generate_rsa_key_pair()?;
            ok(Response::Success {
                request_id: request.request_id.clone(),
                response: CommandResponse::GenerateWalletResponse {
                    private_key: HexString::try_from(priv_k)?.0,
                    public_key: HexString::try_from(pub_k)?.0,
                }
            })
        },
        CommandRequest::PrintBalances => {
            let balances =
                    blockchain.lock().unwrap()
                        .all_balances()
                        .iter()
                        .map(|(k, v)| (shorten_long_string(&k.0 .0), v.clone()))
                        .collect();

            ok(Response::Success {
                request_id: request.request_id.clone(),
                response: CommandResponse::PrintBalancesResponse { balances },
            })
        },
        
        CommandRequest::BalanceTransaction { from, to, amount } => {
            let blockchain = blockchain.lock().unwrap();
            let balanced_transaction = &Transaction::new(&PublicKeyStr::from_str(from), &PublicKeyStr::from_str(to), *amount)
                .balance_transaction(&blockchain)?;
            let cbor_bytes = balanced_transaction.to_cbor()?;
            let cbor = hex::encode(&cbor_bytes);
            let body = serde_json::to_string_pretty(balanced_transaction)?;

            ok(Response::Success {
                request_id: request.request_id.clone(),
                response: CommandResponse::BalanceTransactionResponse { request_id: request.request_id.clone(), body, cbor },
            })
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
                }.to_request();

                requests.push((validator.clone(), request));
            }

            println!("{}", serde_json::to_string_pretty(&block)?);

            ok_with_requests(Response::Success {
                request_id: request.request_id.to_owned(),
                response: CommandResponse::CommitTransactionResponse { blockchain_hash: block.hash.to_owned() },
            }, requests)
        },
        CommandRequest::RequestTransactionValidation { blockchain_previous_tip, blockchain_new_tip, transaction_cbor, validator_signature: sender_validator_signature, validator } => {
            let mut blockchain = blockchain.lock().unwrap();
            let blockchain_hash = blockchain.blockchain_hash()?;
            if *blockchain_previous_tip != blockchain_hash {
                let msg = format!("Transaction can't be applied for blockchains are not in sync: {} != {}", blockchain_previous_tip, blockchain_hash);
                println!("{}", msg);
                return ok(Response::Error { msg });
            }

            let signed_transaction = SignedBalancedTransaction::try_from(&Cbor::new(&transaction_cbor))?;
            let block = signed_transaction.commit(&mut blockchain, &configuration.validator_private_key)?;
            let blockchain_hash = blockchain.blockchain_hash()?;
            let validator_signature = block.validator_signatures.first().ok_or(anyhow!("Transaction wasn't signed by validator"))?;

            if *blockchain_new_tip != block.hash {
                let msg = format!("Blockchain hash is different. Possibility of a hard fork");
                println!("{}", msg);
                return ok(Response::Error { msg })
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

            ok(Response::Success {
                request_id: request.request_id.to_owned(),
                response: CommandResponse::RequestTransactionValidationResponse {
                    new_blockchain_tip: blockchain_hash,
                    validator_public_key: configuration.validator_public_key.0.0.to_owned(),
                    transaction_cbor: transaction_cbor.to_owned(),
                    validator_signature: validator_signature.validator_signature.0.0.to_owned(),
                },
            })
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
            ok(Response::Success {
                request_id: request.request_id.to_owned(),
                response: CommandResponse::PrintBlockchainResponse { blocks },
            })
        },
    }
}

fn ok(response: Response) -> Result<(Response, Vec<(ValidatorReference, Request)>)> {
    Ok((response, Vec::new()))
}

fn ok_with_requests(response: Response, requests: Vec<(ValidatorReference, Request)>) -> Result<(Response, Vec<(ValidatorReference, Request)>)> {
    Ok((response, requests))
}
