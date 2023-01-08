use std::{sync::{Arc, Mutex}, borrow::BorrowMut};

use anyhow::{Result, anyhow};
use protocol::{response::Response, internal::{InternalResponse, CommandResponse}};

use crate::{blockchain::blockchain::BlockChain, model::{PublicKeyStr, Signature}};

pub fn handle_response(blockchain: &Arc<Mutex<BlockChain>>, response: Response) -> Result<()> {
    match response {
        Response::Internal(InternalResponse::Success {request_id, response}) => handle_command_response(blockchain.lock().unwrap().borrow_mut(), &response),
        Response::Internal(InternalResponse::Error {msg}) => Err(anyhow!("Error happened: {}", msg)),
        Response::External(resp) => Err(anyhow!("Only internal responses are supported here")),
    }
}

fn handle_command_response(blockchain: &mut BlockChain, response: &CommandResponse) -> Result<()> {
    match response {
        CommandResponse::OnBoardValidatorResponse { validators } => Ok(()),
        CommandResponse::SynchronizeBlockchainResponse { transaction_cbor, expected_blockchain_hash } => todo!(),
        CommandResponse::RequestTransactionValidationResponse { validator_public_key, validator_signature, .. } => {
            let last = blockchain.blocks.last_mut().unwrap();
            last.validator_signatures.push(super::blockchain::validator_signature::ValidatorSignature::new(&PublicKeyStr::from_str(&validator_public_key), &Signature::from_string(&validator_signature)));
            println!("Validation added {}", serde_json::to_string_pretty(&last.validator_signatures).unwrap());
            Ok(())
        },
    }
}