use anyhow::{anyhow, Result};
use log::debug;
use protocol::{
    common::{Validator, ValidatorWithSignature},
    request::CommandResponse,
    request::{CommandRequest, Request, Response, ResponseBody},
};

use crate::{
    blockchain::{cbor::Cbor, signed_balanced_transaction::SignedBalancedTransaction},
    model::{PublicKeyStr, Signature},
    runtime::{
        configuration::{ValidatorAddress, ValidatorReference},
        validator_runtime::ValidatorRuntime,
    },
};

use super::blockchain::validator_signature::ValidatorSignature;

pub fn handle_response(
    rt: &mut ValidatorRuntime,
    request_id: &str,
    response: &Response,
) -> Result<Vec<(ValidatorReference, Request)>> {
    match response {
        Response {
            orig_request_id,
            replier,
            body: ResponseBody::Success(response),
        } => handle_command(rt, request_id, &replier, response),
        Response {
            orig_request_id,
            replier,
            body: ResponseBody::Error { msg },
        } => Err(anyhow!(msg.to_owned())),
    }
}

fn handle_command(
    rt: &mut ValidatorRuntime,
    request_id: &str,
    replier: &Validator,
    response: &CommandResponse,
) -> Result<Vec<(ValidatorReference, Request)>> {
    match response {
        CommandResponse::OnBoardValidatorResponse {
            on_boarding_validator,
            validators,
            blockchain_tip,
        } => {
            let new_validators: Vec<ValidatorReference> = validators
                .iter()
                .flat_map(|validator| {
                    validator.address.clone().map(|addr| ValidatorReference {
                        pk: PublicKeyStr::from_str(&validator.public_key),
                        address: ValidatorAddress(addr.to_owned()),
                    })
                })
                .collect();
            rt.add_validators(&new_validators[..]);
            debug!(
                "Validators added: {:?}",
                rt.validators
                    .iter()
                    .map(|validator| &validator.address)
                    .collect::<Vec<&ValidatorAddress>>()
            );

            let this_blockchain_tip = rt.blockchain.blockchain_hash()?;
            if this_blockchain_tip != *blockchain_tip {
                let on_boarding_validator = ValidatorReference::try_from(on_boarding_validator)?;
                let command = CommandRequest::RequestSynchronization {
                    blockchain_tip: this_blockchain_tip,
                };
                return ok_with_requests(vec![(
                    on_boarding_validator,
                    command.to_request(&rt.configuration.validator()),
                )]);
            }

            ok()
        }
        CommandResponse::SynchronizeBlockchainResponse {} => ok(),
        CommandResponse::RequestTransactionValidationResponse {
            old_blockchain_tip,
            new_blockchain_tip,
            validator_public_key,
            transaction_cbor,
            validator_signature: _validator_signature,
        } => {
            let last = rt.blockchain.blocks.last_mut().unwrap();
            let validator_signature = ValidatorSignature::new(
                &PublicKeyStr::from_str(&validator_public_key),
                &Signature::from_string(&_validator_signature),
            );
            let validator_signature_json = serde_json::to_string_pretty(&validator_signature)?;
            last.add_validator_signature(validator_signature);
            debug!(
                "New validation added (total {}) {}",
                last.validator_signatures().len(),
                validator_signature_json
            );

            let prev_block = "not needed atm"; // &blockchain.blocks[blockchain.blocks.len() - 2];
            let current_block = &rt.blockchain.blocks[rt.blockchain.blocks.len() - 1];

            let requests = rt
                .validators
                .iter()
                .flat_map(
                    |ValidatorReference {
                         pk: validator_pub_key,
                         address: validator_addr,
                     }| {
                        if *validator_pub_key != rt.configuration.validator_public_key {
                            if let Some(validator_address) =
                                rt.find_validator_address_by_key(validator_pub_key)
                            {
                                let command = CommandRequest::SynchronizeBlockchain {
                                    signatures: vec![ValidatorWithSignature {
                                        validator: Validator {
                                            address: Some(validator_address.0.to_owned()),
                                            public_key: validator_public_key.to_owned(),
                                        },
                                        signature: _validator_signature.to_owned(),
                                    }],
                                    transaction_cbor: transaction_cbor.to_owned(),
                                    blockchain_tip_before_transaction: prev_block.to_owned(),
                                    blockchain_tip_after_transaction: current_block.hash.to_owned(),
                                };
                                debug!(
                                    "Synchronisation request will be sent to {}",
                                    validator_address.0
                                );
                                let request = command.to_request(&rt.configuration.validator());
                                vec![(
                                    ValidatorReference {
                                        pk: validator_pub_key.clone(),
                                        address: validator_address,
                                    },
                                    request,
                                )]
                            } else {
                                debug!(
                                    "Validator {} is not registered with this node",
                                    validator_pub_key
                                );
                                Vec::new()
                            }
                        } else {
                            Vec::new()
                        }
                    },
                )
                .collect();

            ok_with_requests(requests)
        }

        CommandResponse::RequestSynchronizationResponse {
            previous_hash,
            next_hash,
            transaction_cbor,
            signatures,
        } => {
            debug!(
                "Processing RequestSynchronizationResponse. Base hash {}, expected hash {}",
                previous_hash, next_hash
            );
            let current_blockchain_tip = rt.blockchain.blockchain_hash()?;
            if current_blockchain_tip != *previous_hash {
                debug!("RequestSynchronizationResponse is impossible because base hash from the requester {} does not match to base hash of the receiver {}", previous_hash, current_blockchain_tip);
                return ok();
            }

            let signed_transaction =
                SignedBalancedTransaction::try_from(&Cbor::new(&transaction_cbor))?;
            let block = signed_transaction
                .commit(&mut rt.blockchain, &rt.configuration.validator_private_key)?;
            let validator_signature = block
                .validator_signatures()
                .first()
                .ok_or(anyhow!("Transaction wasn't signed by validator"))?;

            debug!("Transaction applied, new block hash is {}", block.hash);

            if block.hash.to_owned() != *next_hash {
                debug!("RequestSynchronizationResponse failed because resulting hash {} does not match to expected hash {}", block.hash, next_hash);
                return ok();
            }

            let synchronisaction_command = CommandRequest::AddValidatorSignature {
                hash: block.hash.to_owned(),
                validator_signature: ValidatorWithSignature::from(validator_signature),
            };

            let command = CommandRequest::RequestSynchronization {
                blockchain_tip: block.hash.to_owned(),
            };
            let request = command.to_request(&rt.configuration.validator());

            ok_with_requests(vec![
                (ValidatorReference::try_from(replier)?, request),
                (
                    ValidatorReference::try_from(replier)?,
                    synchronisaction_command.to_request(&rt.configuration.validator()),
                ),
            ])
        }
        CommandResponse::Nothing => ok(),
        CommandResponse::PrintValidatorsResponse(_) => {
            err_client_command_used_by_node("PrintValidatorsResponse")
        }

        client_command => err_client_command_used_by_node(&format!("{:?}", client_command)),
    }
}

fn err_client_command_used_by_node<T>(msg: &str) -> Result<T> {
    Err(anyhow::anyhow!(
        "This command {} is not supposed to be used by validator",
        msg
    ))
}

fn ok() -> Result<Vec<(ValidatorReference, Request)>> {
    Ok(Vec::new())
}

fn ok_with_requests(
    requests: Vec<(ValidatorReference, Request)>,
) -> Result<Vec<(ValidatorReference, Request)>> {
    Ok(requests)
}
