use std::{thread, sync::{Mutex, Arc}};

use protocol::{
    external::{ExternalRequest, ExternalResponse, UserCommand, UserCommandResponse},
    internal::{CommandResponse, InternalRequest, InternalResponse, CommandRequest, Validator, ValidatorSignature},
};
use serde::Serialize;

use crate::{
    configuration::{Configuration, ValidatorAddress},
    encryption::generate_rsa_key_pair,
    model::{HexString, PublicKeyStr}, blockchain::{blockchain::BlockChain, transaction::Transaction, signed_balanced_transaction::SignedBalancedTransaction, cbor::Cbor, block::Block}, client::Client, utils::shorten_long_string,
};
use anyhow::{Result, anyhow};

pub trait RequestHandler<T: Serialize> {
    type RESPONSE;
    fn handle_request(
        &self,
        blockchain: Arc<Mutex<BlockChain>>,
        configuration: &mut Configuration,
    ) -> Result<Self::RESPONSE>;
}

impl RequestHandler<InternalResponse> for InternalRequest {
    type RESPONSE = InternalResponse;
    fn handle_request(
        &self,
        blockchain: Arc<Mutex<BlockChain>>,
        configuration: &mut Configuration,
    ) -> Result<Self::RESPONSE> {
        match &self.command {
            protocol::internal::CommandRequest::OnBoardValidator { return_address, public_key, retransmitted } => {

                // to avoid infinite loop (seek for better solution :-/)
                if !retransmitted {
                    for (validator_pub_key, validator_addr) in &configuration.validators {
                        let return_address_ = return_address.clone();
                        let validator_addr_ = validator_addr.clone();
                        let public_key_ = public_key.clone();
                        thread::spawn(move || {
                            let client = Client::new(&validator_addr_.0);
                            client.register_validator(&return_address_, &PublicKeyStr::from_str(&public_key_), true).unwrap()
                        });
                    }
                }

                let validator_address = ValidatorAddress(return_address.to_owned());
                configuration.add_validators(&[(PublicKeyStr::from_str(&public_key), validator_address)]);
                println!(
                    "Added new validator {:?}, total validators {}",
                    return_address,
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

                Ok(InternalResponse::Success {
                    request_id: self.request_id.to_owned(),
                    response: CommandResponse::OnBoardValidatorResponse { validators: all_validators },
                })
            },
            CommandRequest::SynchronizeBlockchain { address, blockchain_hash } => todo!(),
            CommandRequest::RequestTransactionValidation { blockchain_previous_tip, blockchain_new_tip, transaction_cbor, validator_signature } => {
                let blockchain_hash = blockchain.lock().unwrap().blockchain_hash()? ;
                if *blockchain_previous_tip != blockchain_hash {
                    let msg = format!("Transaction can't be applied for blockchains are not in sync: {} != {}", blockchain_previous_tip, blockchain_hash);
                    println!("{}", msg);
                    return Ok(InternalResponse::Error { msg });
                }

                let signed_transaction = SignedBalancedTransaction::try_from(&Cbor::new(&transaction_cbor))?;
                let block = signed_transaction.commit(&blockchain, &configuration.validator_private_key)?;
                let blockchain_hash = blockchain.lock().unwrap().blockchain_hash()?;
                let validator_signature = block.validator_signatures.first().ok_or(anyhow!("Transaction wasn't signed by validator"))?;

                if *blockchain_new_tip != block.hash {
                    let msg = format!("Blockchain hash is different. Possibility of a hard fork");
                    println!("{}", msg);
                    return Ok(InternalResponse::Error { msg })
                }

                println!("Transaction successfully verified and added to blockchain. Total verifications: {}", block.validator_signatures.len());
                println!("{}", serde_json::to_string_pretty(&block)?);

                Ok(InternalResponse::Success {
                    request_id: self.request_id.to_owned(),
                    response: CommandResponse::RequestTransactionValidationResponse {
                        new_blockchain_tip: blockchain_hash,
                        validator_public_key: configuration.validator_public_key.0.0.to_owned(),
                        transaction_cbor: transaction_cbor.to_owned(),
                        validator_signature: validator_signature.validator_signature.0.0.to_owned(),
                    },
                })
            },
        }
    }
}

impl RequestHandler<ExternalResponse> for ExternalRequest {
    type RESPONSE = ExternalResponse;
    fn handle_request(
        &self,
        blockchain: Arc<Mutex<BlockChain>>,
        configuration: &mut Configuration,
    ) -> Result<Self::RESPONSE> {
        println!("External request received");
        match &self.command {
            UserCommand::CreateRecord { data } => panic!("Not ready yet"),
            UserCommand::PingCommand { msg } => {
                println!("Received ping command");
                Ok(ExternalResponse::Success(
                    UserCommandResponse::PingCommandResponse {
                        request_id: self.request_id.clone(),
                        msg: format!("Original message: {}, PONG PONG", msg),
                    },
                ))
            },
            UserCommand::GenerateWallet => {
                let (priv_k, pub_k) = &generate_rsa_key_pair()?;
                Ok(ExternalResponse::Success(
                    UserCommandResponse::GenerateWalletResponse {
                        private_key: HexString::try_from(priv_k)?.0,
                        public_key: HexString::try_from(pub_k)?.0,
                    },
                ))
            },
            UserCommand::PrintBalances => {
                if let Ok(blockchain) = blockchain.lock() {
                    let balances: Vec<_> = Vec::from_iter(
                        blockchain
                            .all_balances()
                            .iter()
                            .map(|(k, v)| (shorten_long_string(&k.0 .0), v.clone())),
                    );
    
                    Ok(ExternalResponse::Success(
                        UserCommandResponse::PrintBalancesResponse { balances },
                    ))
                } else {
                    todo!()
                }
            },
            
            UserCommand::BalanceTransaction { from, to, amount } => {
                let balanced_transaction = &Transaction::new(&PublicKeyStr::from_str(from), &PublicKeyStr::from_str(to), *amount)
                    .balance_transaction(&blockchain)?;
                let cbor_bytes = balanced_transaction.to_cbor()?;
                let cbor = hex::encode(&cbor_bytes);
                let body = serde_json::to_string_pretty(balanced_transaction)?;

                Ok(ExternalResponse::Success(
                    UserCommandResponse::BalanceTransactionResponse { request_id: self.request_id.clone(), body, cbor },
                ))
            },

            UserCommand::CommitTransaction { signed_transaction_cbor } => {
                let blockchain_previous_tip = blockchain.lock().unwrap().blockchain_hash()?;
                let signed_transaction = SignedBalancedTransaction::try_from(&Cbor::new(&signed_transaction_cbor))?;
                let block = signed_transaction.commit(&blockchain, &configuration.validator_private_key)?;
                let validator_signature = block.validator_signatures.first().unwrap();

                // if let Some((_, validator_address)) = configuration.validators.first() {
                for (_, validator_address) in &configuration.validators {
                    let client = Client::new(&validator_address.0);
                    client.request_transaction_validation(
                        blockchain.clone(), 
                        &blockchain_previous_tip,
                        &block.hash, 
                        signed_transaction_cbor, 
                        &ValidatorSignature { 
                            validator: Validator { 
                                address: configuration.address(), 
                                public_key: configuration.validator_public_key.0.0.to_owned() 
                            }, 
                            signature: validator_signature.validator_signature.0.0.to_owned() 
                        });
                }

                println!("{}", serde_json::to_string_pretty(&block)?);

                Ok(ExternalResponse::Success(
                    UserCommandResponse::CommitTransactionResponse { blockchain_hash: block.hash.to_owned() },
                ))
            },
            UserCommand::PrintBlockchain => {
                let blockchain = blockchain.lock().unwrap();
                let blocks = blockchain.blocks.iter().map(|block| {
                    let block_str = String::new();
                    
                    block_str
                }).collect();
                Ok(ExternalResponse::Success(
                    UserCommandResponse::PrintBlockchainResponse { blocks },
                ))
            },
        }
    }
}
