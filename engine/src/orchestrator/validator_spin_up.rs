use crate::{model::requests::{Request, ValidatorStartedRequest, CommandResponse, ValidatorStartedResponse, ResponseWithRequests, Response, InternalRequest, CommandRequest, RegisterRemoteValidatorRequest}, runtime::validator_runtime::ValidatorRuntime};
use anyhow::Result;
use log::debug;

use super::request_handler::RequestHandler;

impl RequestHandler for ValidatorStartedRequest {
    fn handle_request(&self, request: &Request, rt: & mut ValidatorRuntime) -> Result<(CommandResponse, Vec<InternalRequest>)> {
        let response = CommandResponse::ValidatorStarted(ValidatorStartedResponse);
        let this_validator_reference = rt.configuration.validator_ref();
        let this_validator = rt.configuration.validator();
        debug!("Total validators to send request to: {}", rt.validators.len());
        let requests: Vec<_> = rt.validators.iter().map(|remote_validator_ref| {
            CommandRequest::RegisterRemoteValidator(RegisterRemoteValidatorRequest {validator: this_validator_reference.clone()})
                .to_internal_request(&this_validator, remote_validator_ref)
        }).collect();
        Ok((response, requests))
    }
}