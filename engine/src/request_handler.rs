use protocol::{
    external::{ExternalRequest, ExternalResponse, UserCommand, UserCommandResponse},
    internal::{CommandResponse, InternalRequest, InternalResponse, CommandRequest}, response::Response::Internal,
};
use serde::Serialize;

use crate::{
    configuration::{Configuration, NodeType, ValidatorAddress},
    encryption::generate_rsa_key_pair,
    model::{HexString, PublicKeyStr, Signature}, blockchain::{blockchain::BlockChain, transaction::Transaction, balanced_transaction::BalancedTransaction, signed_balanced_transaction::SignedBalancedTransaction, cbor::Cbor}, client::send_bytes,
};
use anyhow::{anyhow, Result};

pub trait RequestHandler<T: Serialize> {
    type RESPONSE;
    fn handle_request(
        &self,
        blockchain: &mut BlockChain,
        configuration: &mut Configuration,
    ) -> Result<Self::RESPONSE>;
}

impl RequestHandler<InternalResponse> for InternalRequest {
    type RESPONSE = InternalResponse;
    fn handle_request(
        &self,
        blockchain: &mut BlockChain,
        configuration: &mut Configuration,
    ) -> Result<Self::RESPONSE> {
        match &self.command {
            protocol::internal::CommandRequest::OnBoardValidator { return_address } => {
                match configuration.node_type {
                    NodeType::Coordinator { ref mut validators } => {
                        let validator_address = ValidatorAddress(return_address.to_owned());
                        validators.push(validator_address);
                        println!(
                            "Added new validator {:?}, total validators {}",
                            return_address,
                            &validators.len()
                        );
                        Ok(())
                    }
                    NodeType::Validator => {
                        Err(anyhow!("Validator can't on-board another validator"))
                    }
                }?;

                Ok(InternalResponse::Success {
                    request_id: self.request_id.to_owned(),
                    response: CommandResponse::OnBoardValidatorResponse,
                })
            },
            protocol::internal::CommandRequest::ValidateAndCommitTransaction {
                from,
                to,
                amount,
                signature,
            } => {
                if configuration.node_type != NodeType::Validator {
                    return Err(anyhow!("Node has to be a validator"));
                }

                Ok(InternalResponse::Success {
                    request_id: self.request_id.to_owned(),
                    response: CommandResponse::OnBoardValidatorResponse,
                })
            },
            protocol::internal::CommandRequest::CommitTransaction { signed_transaction_cbor } => {
                let signed_transaction = SignedBalancedTransaction::try_from(&Cbor::new(&signed_transaction_cbor))?;
                signed_transaction.commit(blockchain, &configuration.validator_private_key)?;
                let blockchain_hash = blockchain.blockchain_hash()?;

                Ok(InternalResponse::Success {
                    request_id: self.request_id.to_owned(),
                    response: CommandResponse::CommitTransactionResponse { blockchain_hash },
                })
            },
            CommandRequest::SynchronizeBlockchain { address, blockchain_hash } => todo!(),
        }
    }
}

impl RequestHandler<ExternalResponse> for ExternalRequest {
    type RESPONSE = ExternalResponse;
    fn handle_request(
        &self,
        blockchain: &mut BlockChain,
        configuration: &mut Configuration,
    ) -> Result<Self::RESPONSE> {
        println!("External request received");
        match (&self.command, &configuration.node_type) {
            (UserCommand::CreateRecord { data }, _) => panic!("Not ready yet"),
            (UserCommand::PingCommand { msg }, _) => {
                println!("Received ping command");
                Ok(ExternalResponse::Success(
                    UserCommandResponse::PingCommandResponse {
                        request_id: self.request_id.clone(),
                        msg: format!("Original message: {}, PONG PONG", msg),
                    },
                ))
            },
            (UserCommand::GenerateWallet, _) => {
                let (priv_k, pub_k) = &generate_rsa_key_pair()?;
                Ok(ExternalResponse::Success(
                    UserCommandResponse::GenerateWalletResponse {
                        private_key: HexString::try_from(priv_k)?.0,
                        public_key: HexString::try_from(pub_k)?.0,
                    },
                ))
            },
            (UserCommand::PrintBalances, _) => {
                let shorten_public_address = |str: &str| {
                    let mut res = String::new();
                    res += &str[0..10];
                    res += "....";
                    res += &str[str.len() - 10..str.len()];
                    res.to_string()
                };

                let balances = Vec::from_iter(
                    blockchain
                        .all_balances()
                        .iter()
                        .map(|(k, v)| (shorten_public_address(&k.0 .0), v.clone())),
                );

                Ok(ExternalResponse::Success(
                    UserCommandResponse::PrintBalancesResponse { balances },
                ))
            },
            
            (UserCommand::BalanceTransaction { from, to, amount }, _) => {
                let balanced_transaction = &Transaction::new(&PublicKeyStr::from_str(from), &PublicKeyStr::from_str(to), *amount).balance_transaction(blockchain)?;
                let cbor_bytes = balanced_transaction.to_cbor()?;
                let cbor = hex::encode(&cbor_bytes);
                let body = serde_json::to_string_pretty(balanced_transaction)?;

                Ok(ExternalResponse::Success(
                    UserCommandResponse::BalanceTransactionResponse { request_id: self.request_id.clone(), body, cbor },
                ))
            },

            (UserCommand::CommitTransaction { signed_transaction_cbor }, NodeType::Coordinator { validators }) => {

                let mut precommit_results: Vec<(&ValidatorAddress, String)> = Vec::new();

                for validator in validators {
                    let response = send_bytes(&validator.0, CommandRequest::CommitTransaction { signed_transaction_cbor: signed_transaction_cbor.to_owned() }.to_request())?;
                    match response {
                        Internal(InternalResponse::Success { request_id, response: CommandResponse::CommitTransactionResponse { blockchain_hash } }) => {
                            precommit_results.push((validator, blockchain_hash));
                        },
                        response => {
                            println!("Error happened. Received wrong response for PreCommitTransaction request: {:?}", response);
                        },
                    }
                }

                Ok(ExternalResponse::Success(
                    UserCommandResponse::CommitTransactionResponse { transaction_id: "".to_owned() },
                ))
            },

            (cmd, node_type) => {
                Ok(ExternalResponse::Error { msg: format!("Unsupported command {:?} for node type {:?}", cmd, node_type) })
            }
        }
    }
}
