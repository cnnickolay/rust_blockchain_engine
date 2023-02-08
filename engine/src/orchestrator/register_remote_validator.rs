use anyhow::Result;

use crate::{model::requests::{RegisterRemoteValidatorRequest, Request, CommandResponse, InternalRequest, RegisterRemoteValidatorResponse}, runtime::validator_runtime::ValidatorRuntime};

use super::request_handler::RequestHandler;

impl RequestHandler for RegisterRemoteValidatorRequest {
    fn handle_request(&self, request: &Request, rt: & mut ValidatorRuntime) -> Result<(CommandResponse, Vec<InternalRequest>)> {
        println!("Received a request from {:?}", request.command);
        Ok((CommandResponse::RegisterRemoteValidator(RegisterRemoteValidatorResponse), Vec::new()))
    }
}