use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::common::{Validator, ValidatorWithSignature};

#[derive(Serialize, Deserialize, Debug)]
pub struct Request {
    pub request_id: String,
    pub parent_request_id: Option<String>,
    // sender is none if request was done by a client, not a validator
    pub sender: Option<Validator>,
    pub command: CommandRequest,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Response {
    pub orig_request_id: String,
    pub replier: Validator,
    pub body: ResponseBody,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ResponseBody {
    Success(CommandResponse),
    Error { msg: String },
}

#[derive(Serialize, Deserialize, Debug)]
pub enum CommandRequest {
    PingCommand {
        msg: String,
    },
    GenerateWallet,
    PrintBalances,
    PrintValidators,
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
        signatures: Vec<ValidatorWithSignature>,
        transaction_cbor: String,
        blockchain_tip_before_transaction: String,
        blockchain_tip_after_transaction: String,
    },
    RequestTransactionValidation {
        // blockchain hash before transaction was committed
        blockchain_previous_tip: String,
        // blockchain hash after transaction was committed
        blockchain_new_tip: String,
        transaction_cbor: String,
        validator_signature: ValidatorWithSignature,
        validator: Validator,
    },
    RequestSynchronization {
        blockchain_tip: String,
    },
    AddValidatorSignature {
        hash: String,
        validator_signature: ValidatorWithSignature,
    },
    BlockchainTip,
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
    PrintValidatorsResponse(_PrintValidatorsResponse),
    BalanceTransactionResponse {
        request_id: String,
        body: String,
        cbor: String,
    },
    CommitTransactionResponse {
        blockchain_hash: String,
    },
    PrintBlockchainResponse {
        blocks: Vec<String>,
    },
    OnBoardValidatorResponse {
        on_boarding_validator: Validator,
        validators: Vec<Validator>,
        blockchain_tip: String,
    },
    SynchronizeBlockchainResponse {},
    RequestTransactionValidationResponse {
        // blockchain hash before the transaction was applied
        old_blockchain_tip: String,
        // blockchain hash after the transaction was applied
        new_blockchain_tip: String,
        validator_public_key: String,
        transaction_cbor: String,
        validator_signature: String,
    },
    RequestSynchronizationResponse {
        previous_hash: String,
        next_hash: String,
        transaction_cbor: String,
        signatures: Vec<ValidatorWithSignature>,
    },
    Nothing,
    BlockchainTipResponse {
        blockchain_tip_hash: String,
    },
}

#[derive(Serialize, Deserialize, Debug)]
pub struct _PrintValidatorsResponse {
    pub validators: Vec<Validator>,
}

// ################

impl CommandResponse {
    pub fn name(&self) -> String {
        match self {
            CommandResponse::PingCommandResponse { .. } => "PingCommandResponse".to_owned(),
            CommandResponse::GenerateWalletResponse { .. } => "GenerateWalletResponse".to_owned(),
            CommandResponse::PrintBalancesResponse { .. } => "PrintBalancesResponse".to_owned(),
            CommandResponse::PrintValidatorsResponse(..) => "PrintValidatorsResponse".to_owned(),
            CommandResponse::BalanceTransactionResponse { .. } => {
                "BalanceTransactionResponse".to_owned()
            }
            CommandResponse::CommitTransactionResponse { .. } => {
                "CommitTransactionResponse".to_owned()
            }
            CommandResponse::PrintBlockchainResponse { .. } => "PrintBlockchainResponse".to_owned(),
            CommandResponse::OnBoardValidatorResponse { .. } => {
                "OnBoardValidatorResponse".to_owned()
            }
            CommandResponse::SynchronizeBlockchainResponse {} => {
                "SynchronizeBlockchainResponse".to_owned()
            }
            CommandResponse::RequestTransactionValidationResponse { .. } => {
                "RequestTransactionValidationResponse".to_owned()
            }
            CommandResponse::RequestSynchronizationResponse { .. } => {
                "RequestSynchronizationResponse".to_owned()
            }
            CommandResponse::Nothing => "Nothing".to_owned(),
            CommandResponse::BlockchainTipResponse { .. } => "BlockchainTipResponse".to_owned(),
        }
    }
}

impl CommandRequest {
    pub fn name(&self) -> String {
        match self {
            CommandRequest::PingCommand { .. } => "PingCommand".to_owned(),
            CommandRequest::GenerateWallet => "GenerateWallet".to_owned(),
            CommandRequest::PrintBalances => "PrintBalances".to_owned(),
            CommandRequest::PrintValidators => "PrintValidators".to_owned(),
            CommandRequest::BalanceTransaction { .. } => "BalanceTransaction".to_owned(),
            CommandRequest::CommitTransaction { .. } => "CommitTransaction".to_owned(),
            CommandRequest::PrintBlockchain => "PrintBlockchain".to_owned(),
            CommandRequest::OnBoardValidator { .. } => "OnBoardValidator".to_owned(),
            CommandRequest::SynchronizeBlockchain { .. } => "SynchronizeBlockchain".to_owned(),
            CommandRequest::RequestTransactionValidation { .. } => {
                "RequestTransactionValidation".to_owned()
            }
            CommandRequest::RequestSynchronization { .. } => "RequestSynchronization".to_owned(),
            CommandRequest::AddValidatorSignature { .. } => "AddValidatorSignature".to_owned(),
            CommandRequest::BlockchainTip => "BlockchainTip".to_owned(),
        }
    }

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

    pub fn to_client_request(self) -> Request {
        Request::new_for_client(self)
    }

    pub fn to_request(self, sender: &Validator) -> Request {
        Request::new(sender, self)
    }

    pub fn to_request_with_id(self, validator: Validator, request_id: &str) -> Request {
        Request {
            sender: Some(validator),
            request_id: request_id.to_owned(),
            command: self,
            parent_request_id: None,
        }
    }
}

impl Request {
    pub fn new_for_client(command: CommandRequest) -> Self {
        let request_id = Uuid::new_v4().to_string();
        Self {
            sender: None,
            request_id,
            command,
            parent_request_id: None,
        }
    }
    pub fn new(sender: &Validator, command: CommandRequest) -> Self {
        let request_id = Uuid::new_v4().to_string();
        Self {
            sender: Some(sender.clone()),
            request_id,
            command,
            parent_request_id: None,
        }
    }
    pub fn new_with_id(sender: &Validator, command: CommandRequest, request_id: &str) -> Self {
        Self {
            sender: Some(sender.clone()),
            request_id: request_id.to_string(),
            command,
            parent_request_id: None,
        }
    }
}
