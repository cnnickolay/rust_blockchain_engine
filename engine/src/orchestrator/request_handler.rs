use crate::{model::requests::{Request, Response, CommandResponse, ResponseWithRequests, InternalRequest}, runtime::validator_runtime::ValidatorRuntime};
use anyhow::Result;

pub trait RequestHandler {
    fn handle_request(&self, request: &Request, rt: & mut ValidatorRuntime) -> Result<(CommandResponse, Vec<InternalRequest>)>;
}