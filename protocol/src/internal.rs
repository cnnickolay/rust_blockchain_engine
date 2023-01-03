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
        public_key: String,
        return_address: String,
        retransmitted: bool
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
        validators: Vec<Validator>
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

#[derive(Serialize, Deserialize, Debug)]
pub struct Validator {
    pub address: String,
    pub public_key: String
}

// ################

impl CommandRequest {
    pub fn new_on_board_command(return_address: &str, public_key: &str, retransmitted: bool) -> CommandRequest {
        CommandRequest::OnBoardValidator {
            return_address: return_address.to_owned(),
            public_key: public_key.to_owned(),
            retransmitted
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
