pub mod validator_spin_up;
pub mod request_handler;
pub mod register_remote_validator;

use super::client_wrappers::ClientWrapper;
use crate::{runtime::{
        validator_runtime::ValidatorRuntime,
        validator_state::ValidatorState::{Election, Expanse, StartUp},
    }, model::requests::{Request, CommandRequest, CommandResponse, RegisterRemoteValidatorResponse, ResponseWithRequests, Response, ResponseBody, InternalRequest}, client_wrappers::ClientWrapperImpl, orchestrator::request_handler::RequestHandler};
use anyhow::{Result, anyhow};
use log::{error, info, debug};

pub struct RequestProcessor {
    pub client: Box<dyn ClientWrapper + Send + Sync +'static>,
}

impl RequestProcessor {
    pub fn prod() -> RequestProcessor {
        RequestProcessor {
            client: Box::new(ClientWrapperImpl),
        }
    }

    pub fn new(client_wrapper: Box<dyn ClientWrapper + Send + Sync + 'static>) -> RequestProcessor {
        RequestProcessor { client: client_wrapper }
    }

    pub fn next_request(&self, request: &Request, rt: &mut ValidatorRuntime) -> Result<ResponseWithRequests> {
        let validator_ref = rt.configuration.validator_ref();
        let cmd_to_response = |(cmd, internal_reqs): (CommandResponse, Vec<InternalRequest>)| -> Result<ResponseWithRequests> {
            Ok(cmd.to_ok_response(&request.request_id, validator_ref).with_requests(internal_reqs))
        };

        match rt.state {
            StartUp => {
                match &request.command {
                    CommandRequest::RegisterRemoteValidator(cmd) => {
                        let response = CommandResponse::RegisterRemoteValidator(RegisterRemoteValidatorResponse);
                        cmd_to_response(cmd.handle_request(request, rt)?)
                    },
                    CommandRequest::BlockchainTip(_) => todo!(),
                    CommandRequest::ValidatorStarted(cmd) => {
                        info!("Initializing validator");
                        cmd_to_response(cmd.handle_request(request, rt)?)
                    },
                }
            },
            Election => todo!(),
            Expanse => todo!(),
        }
    }

    pub fn process_response(&self, request: &Request, response: &Response, rt: &mut ValidatorRuntime) -> Result<Vec<InternalRequest>> {
        match &response.body {
            ResponseBody::Success(response) => {
                match response {
                    CommandResponse::RegisterRemoteValidator(_) => {
                        debug!("Remote validator registered successfully");
                        Ok(Vec::new())
                    },
                    CommandResponse::BlockchainTip(_) => todo!(),
                    CommandResponse::ValidatorStarted(_) => Err(anyhow!("ValidatorStarted should never be received as a reply"))
                }
            },
            ResponseBody::Error { msg } => {
                error!("Error happened while processing request {:?}: {}", request, msg);
                Ok(Vec::new())
            }
        }
    }

    fn synchronize(&self, rt: &ValidatorRuntime) -> Result<()> {
        // 1. find out which blockchain is the dominant on the network (>50% of network should share it)
        let sender = rt.configuration.validator();
        for validator in &rt.validators {
            let blockchain_tip = self
                .client
                .send_blockchain_tip_request(&validator.address, &sender)?;
        }

        // 2. synchronize with these nodes
        Ok(())
    }
}
