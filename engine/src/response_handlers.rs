use std::{sync::{Arc, Mutex}, borrow::BorrowMut};

use anyhow::Result;
use protocol::{request::CommandResponse, request::{Request, Response}};

use crate::{blockchain::blockchain::BlockChain, model::{PublicKeyStr, Signature}, configuration::{ValidatorReference, ValidatorAddress, Configuration}};

use super::blockchain::validator_signature::ValidatorSignature;

pub fn handle_response(blockchain: &Arc<Mutex<BlockChain>>, configuration: &mut Configuration, response: &Response) -> Result<Vec<(ValidatorReference, Request)>> {
    match response {
        Response::Success { request_id, response } => handle_command(blockchain.lock().unwrap().borrow_mut(), configuration, response),
        Response::Error { msg } => Err(anyhow::anyhow!(msg.to_owned())),
    }
}

fn handle_command(blockchain: &mut BlockChain, configuration: &mut Configuration, response: &CommandResponse) -> Result<Vec<(ValidatorReference, Request)>> {
    match response {
        CommandResponse::OnBoardValidatorResponse { validators } => {
            let new_validators: Vec<_> = validators.iter().map(|v| (PublicKeyStr::from_str(&v.public_key), ValidatorAddress(v.address.to_owned()))).collect();
            configuration.add_validators(&new_validators);
            println!("Validators added: {:?}", configuration.validators.iter().map(|validator| &validator.1).collect::<Vec<&ValidatorAddress>>());

            Ok(Vec::new())
        },
        CommandResponse::SynchronizeBlockchainResponse { transaction_cbor, expected_blockchain_hash } => todo!(),
        CommandResponse::RequestTransactionValidationResponse { validator_public_key, validator_signature, .. } => {
            let last = blockchain.blocks.last_mut().unwrap();
            let validator_signature = ValidatorSignature::new(&PublicKeyStr::from_str(&validator_public_key), &Signature::from_string(&validator_signature));
            let validator_signature_json = serde_json::to_string_pretty(&validator_signature)?;
            last.validator_signatures.push(validator_signature);
            println!("New validation added (total {}) {}", last.validator_signatures.len(), validator_signature_json);
            Ok(Vec::new())
        },
        CommandResponse::PingCommandResponse { msg } => todo!(),
        CommandResponse::GenerateWalletResponse { private_key, public_key } => todo!(),
        CommandResponse::PrintBalancesResponse { balances } => todo!(),
        CommandResponse::BalanceTransactionResponse { request_id, body, cbor } => todo!(),
        CommandResponse::CommitTransactionResponse { blockchain_hash } => todo!(),
        CommandResponse::PrintBlockchainResponse { blocks } => todo!(),
    }
}