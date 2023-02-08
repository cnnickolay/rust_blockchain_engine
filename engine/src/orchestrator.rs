use super::client_wrappers::ClientWrapper;
use crate::{runtime::{
        validator_runtime::ValidatorRuntime,
        validator_state::ValidatorState::{Election, Expanse, StartUp},
    }, model::requests::{Request, CommandRequest, CommandResponse, RegisterRemoteValidatorResponse, ResponseWithRequests, Response, ResponseBody, InternalRequest}, client_wrappers::ClientWrapperImpl};
use anyhow::Result;
use log::error;

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
        match rt.state {
            StartUp => {
                match &request.command {
                    CommandRequest::RegisterRemoteValidator(register_validator) => {
                        let response = CommandResponse::RegisterRemoteValidator(RegisterRemoteValidatorResponse)
                            .to_ok_response(&request.request_id, rt.configuration.validator_ref());
                        Ok(response.no_requests())
                    },
                    CommandRequest::BlockchainTip(_) => todo!(),
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
                    CommandResponse::RegisterRemoteValidator(_) => todo!(),
                    CommandResponse::BlockchainTip(_) => todo!(),
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
