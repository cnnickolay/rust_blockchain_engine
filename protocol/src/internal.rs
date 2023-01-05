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
    SynchronizeBlockchain {
        address: String,
        blockchain_hash: String,
    },
    RequestTransactionValidation {
        // blockchain hash before transaction was committed
        blockchain_previous_tip: String,
        // blockchain hash after transaction was committed
        blockchain_new_tip: String,
        transaction_cbor: String,
        validator_signature: ValidatorSignature,
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub enum CommandResponse {
    OnBoardValidatorResponse {
        validators: Vec<Validator>
    },
    SynchronizeBlockchainResponse {
        transaction_cbor: String,
        expected_blockchain_hash: String,
    },
    RequestTransactionValidationResponse {
        new_blockchain_tip: String,
        validator_public_key: String,
        transaction_cbor: String,
        validator_signature: String
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Validator {
    pub address: String,
    pub public_key: String
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ValidatorSignature {
    pub validator: Validator,
    pub signature: String
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
