use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::request::Request;

#[derive(Serialize, Deserialize, Debug)]
pub struct ExternalRequest {
    pub request_id: String,
    pub command: UserCommand,
}

impl ExternalRequest {
    pub fn new(command: UserCommand) -> Self {
        let request_id = Uuid::new_v4().to_string();
        ExternalRequest {
            request_id,
            command,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub enum UserCommand {
    PingCommand { msg: String },
    CreateRecord { data: String },
    GenerateWallet,
    GenerateNonce { address: String },
    PrintBalances,
    Transaction {
        nonce: String,
        from: String,
        to: String,
        amount: u64,
        signature: String,
    }
}

impl UserCommand {
    pub fn new_ping(msg: &str) -> UserCommand {
        UserCommand::PingCommand {
            msg: msg.to_string(),
        }
    }

    pub fn new_generate_nonce(address: &str) -> UserCommand {
        UserCommand::GenerateNonce {
            address: address.to_string(),
        }
    }

    pub fn new_transaction(nonce: &str, from: &str, to: &str, amount: u64, signature: &str) -> UserCommand {
        UserCommand::Transaction {
            nonce: nonce.to_string(),
            from: from.to_string(),
            to: to.to_string(),
            amount: amount,
            signature: signature.to_string(),
        }
    }

    pub fn to_request(self) -> Request {
        Request::External(ExternalRequest::new(self))
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub enum UserCommandResponse {
    PingCommandResponse {
        request_id: String,
        msg: String,
    },
    GenerateWalletResponse {
        private_key: String,
        public_key: String,
    },
    GenerateNonceResponse {
        nonce: String,
    },
    PrintBalancesResponse {
        balances: Vec<(String, u64)>,
    },
    TransactionResponse {
        request_id: String,
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ExternalResponse {
    Success(UserCommandResponse),
    Error { msg: String },
}
