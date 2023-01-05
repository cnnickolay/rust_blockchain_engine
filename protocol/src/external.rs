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
    PrintBalances,
    BalanceTransaction {
        from: String,
        to: String,
        amount: u64,
    },
    CommitTransaction {
        signed_transaction_cbor: String,
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
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ExternalResponse {
    Success(UserCommandResponse),
    Error { msg: String },
}

impl ExternalResponse {
    pub fn success(&self) -> Option<&UserCommandResponse> {
        match self {
            ExternalResponse::Success(r) => Some(r),
            _ => None
        }
    }
}

impl UserCommand {
    pub fn new_ping(msg: &str) -> UserCommand {
        UserCommand::PingCommand {
            msg: msg.to_string(),
        }
    }

    pub fn new_balance_transaction(from: &str, to: &str, amount: u64) -> UserCommand {
        UserCommand::BalanceTransaction {
            from: from.to_string(),
            to: to.to_string(),
            amount,
        }
    }

    pub fn new_commit_transaction(signed_transaction_cbor: &str) -> UserCommand {
        UserCommand::CommitTransaction {
            signed_transaction_cbor: signed_transaction_cbor.to_owned(),
        }
    }

    pub fn to_request(self) -> Request {
        Request::External(ExternalRequest::new(self))
    }
}