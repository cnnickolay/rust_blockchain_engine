use futures::stream::RepeatWith;
use protocol::{common::Validator};
use serde::{Serialize, Deserialize};
use uuid::Uuid;

use crate::runtime::configuration::ValidatorReference;


#[derive(Serialize, Deserialize, Debug)]
pub struct Request {
    pub request_id: String,
    pub parent_request_id: Option<String>,
    // sender is none if request was done by a client, not a validator
    pub sender: Option<Validator>,
    pub command: CommandRequest,
}

impl Request {
    pub fn new(sender: &Validator, command: CommandRequest) -> Self {
        let request_id = Uuid::new_v4().to_string();
        Self {
            sender: Some(sender.clone()),
            request_id,
            command,
            parent_request_id: None,
        }
    }
}

pub struct ResponseWithRequests {
    pub response: Response, 
    pub internal_requests: Vec<InternalRequest>
}

pub struct InternalRequest {
    pub request: Request,
    pub validator_reference: ValidatorReference
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Response {
    pub orig_request_id: String,
    pub replier: ValidatorReference,
    pub body: ResponseBody,
}

impl Response {
    pub fn no_requests(self) -> ResponseWithRequests {
        ResponseWithRequests {
            response: self,
            internal_requests: Vec::new(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ResponseBody {
    Success(CommandResponse),
    Error { msg: String },
}

#[derive(Serialize, Deserialize, Debug)]
pub enum CommandRequest {
    RegisterRemoteValidator(RegisterRemoteValidatorRequest),
    BlockchainTip(BlockchainTipRequest)
}

impl CommandRequest {
    pub fn to_request(self, sender: &Validator) -> Request {
        Request::new(sender, self)
    }

    pub fn name(&self) -> String {
        match self {
            CommandRequest::RegisterRemoteValidator(_) => "RegisterRemoteValidator".to_owned(),
            CommandRequest::BlockchainTip(_) => "BlockchainTip".to_owned(),
        } 
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub enum CommandResponse {
    RegisterRemoteValidator(RegisterRemoteValidatorResponse),
    BlockchainTip(BlockchainTipResponse)
}

impl CommandResponse {
    pub fn name(&self) -> String {
        match self {
            CommandResponse::RegisterRemoteValidator(_) => "RegisterRemoteValidator".to_owned(),
            CommandResponse::BlockchainTip(_) => "BlockchainTip".to_owned(),
        } 
    }

    pub fn to_ok_response(self, original_request_id: &str, validator_reference: ValidatorReference) -> Response {
        Response {
            orig_request_id: original_request_id.to_owned(),
            replier: validator_reference,
            body: ResponseBody::Success(self),
        }
    }
}

// Requests
#[derive(Serialize, Deserialize, Debug)]
pub struct RegisterRemoteValidatorRequest {
    pub validator: ValidatorReference
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BlockchainTipRequest;

// Responses
#[derive(Serialize, Deserialize, Debug)]
pub struct RegisterRemoteValidatorResponse;

#[derive(Serialize, Deserialize, Debug)]
pub struct BlockchainTipResponse {
    pub blockchain_tip_hash: String,
}