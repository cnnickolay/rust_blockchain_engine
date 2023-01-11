use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug)]
pub struct Request {
    pub request_id: String,
    pub command: CommandRequest,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Response {
    Success {
        request_id: String,
        response: CommandResponse,
    },
    Error { msg: String },
}

#[derive(Serialize, Deserialize, Debug)]
pub enum CommandRequest {
    PingCommand { msg: String },
    GenerateWallet,
    PrintBalances,
    BalanceTransaction {
        from: String,
        to: String,
        amount: u64,
    },
    CommitTransaction {
        signed_transaction_cbor: String,
    },
    PrintBlockchain,
    OnBoardValidator {
        public_key: String,
        return_address: String,
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
        validator_signature: ValidatorWithSignature,
        validator: Validator
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub enum CommandResponse {
    PingCommandResponse {
        msg: String,
    },
    GenerateWalletResponse {
        private_key: String,
        public_key: String,
    },
    PrintBalancesResponse {
        balances: Vec<(String, u64)>,
    },
    BalanceTransactionResponse {
        request_id: String,
        body: String,
        cbor: String,
    },
    CommitTransactionResponse {
        blockchain_hash: String,
    },
    PrintBlockchainResponse {
        blocks: Vec<String>
    },
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
pub struct ValidatorWithSignature {
    pub validator: Validator,
    pub signature: String
}

// ################

impl CommandRequest {
    pub fn new_ping(msg: &str) -> Self {
        Self::PingCommand {
            msg: msg.to_string(),
        }
    }

    pub fn new_balance_transaction(from: &str, to: &str, amount: u64) -> Self {
        Self::BalanceTransaction {
            from: from.to_string(),
            to: to.to_string(),
            amount,
        }
    }

    pub fn new_commit_transaction(signed_transaction_cbor: &str) -> Self {
        Self::CommitTransaction {
            signed_transaction_cbor: signed_transaction_cbor.to_owned(),
        }
    }

    pub fn new_on_board_command(return_address: &str, public_key: &str) -> CommandRequest {
        CommandRequest::OnBoardValidator {
            return_address: return_address.to_owned(),
            public_key: public_key.to_owned(),
        }
    }

    pub fn to_request(self) -> Request {
        Request::new(self)
    }

    pub fn to_request_with_id(self, request_id: &str) -> Request {
        Request { request_id: request_id.to_owned(), command: self }
    }
}

impl Request {
    pub fn new(command: CommandRequest) -> Self {
        let request_id = Uuid::new_v4().to_string();
        Self {
            request_id,
            command,
        }
    }
    pub fn new_with_id(command: CommandRequest, request_id: &str) -> Self {
        Self {
            request_id: request_id.to_string(),
            command,
        }
    }
}
