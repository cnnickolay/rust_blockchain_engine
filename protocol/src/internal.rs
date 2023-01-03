use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::request::Request;

#[derive(Serialize, Deserialize, Debug)]
pub struct InternalRequest {
    pub request_id: String,
    pub command: CommandRequest,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum InternalResponse {
    Success {
        request_id: String,
        response: CommandResponse,
    },
    Error { msg: String },
}

#[derive(Serialize, Deserialize, Debug)]
pub enum CommandRequest {
    OnBoardValidator {
        return_address: String,
    },
    ValidateAndCommitTransaction {
        from: String,
        to: String,
        amount: u64,
        signature: String,
    },
    CommitTransaction {
        signed_transaction_cbor: String    
    },
    SynchronizeBlockchain {
        address: String,
        blockchain_hash: String,
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub enum CommandResponse {
    OnBoardValidatorResponse {
        validators: Vec<String>
    },
    ValidateAndCommitTransactionResponse,
    CommitTransactionResponse {
        blockchain_hash: String
    },
    SynchronizeBlockchainResponse {
        transaction_cbor: String,
        expected_blockchain_hash: String,
    }
}

// ################

impl CommandRequest {
    pub fn new_on_board_command(return_address: &str) -> CommandRequest {
        CommandRequest::OnBoardValidator {
            return_address: return_address.to_string(),
        }
    }

    pub fn to_request(self) -> Request {
        Request::Internal(InternalRequest::new(self))
    }
}

impl InternalRequest {
    pub fn new(command: CommandRequest) -> Self {
        let request_id = Uuid::new_v4().to_string();
        InternalRequest {
            request_id,
            command,
        }
    }
}
