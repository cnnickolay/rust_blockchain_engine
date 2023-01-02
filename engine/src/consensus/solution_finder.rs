use anyhow::Result;
use crate::blockchain::{blockchain::BlockChain, cbor::Cbor, signed_balanced_transaction::SignedBalancedTransaction};

pub enum IncomingCommand {
    Synchronize { blockchain_hash: String }
}

pub enum OutgoingCommand {
    Synchronize(SynchronizeResponse)
}

pub enum SynchronizeResponse {
    NeedToSync {
        expected_blockchain_hash: String,
        transaction_cbor: Cbor
    },
    InSync
}

pub fn handle_request(blockchain: &BlockChain, incoming_command: IncomingCommand) -> Result<()> {
    match incoming_command {
        IncomingCommand::Synchronize { blockchain_hash } => {
            let transaction = &blockchain.transactions[0];
            send_response(OutgoingCommand::Synchronize(SynchronizeResponse::NeedToSync { expected_blockchain_hash: blockchain_hash, transaction_cbor: transaction.try_into()? }))
        },
    }
}

fn send_response(response: OutgoingCommand) -> Result<()> {
    todo!()
}

fn send_request(request: IncomingCommand) -> Result<()> {
    todo!()
}

pub fn handle_response(blockchain: &mut BlockChain, outgoing_command: OutgoingCommand) -> Result<()> {
    match outgoing_command {
        OutgoingCommand::Synchronize(SynchronizeResponse::NeedToSync { expected_blockchain_hash, ref transaction_cbor }) => {
            let transaction = SignedBalancedTransaction::try_from(transaction_cbor)?;
            let blockchain_hash = blockchain.commit_transaction(&transaction)?;
            if expected_blockchain_hash == blockchain_hash {
                send_request(IncomingCommand::Synchronize { blockchain_hash })?
            }
            Ok(())
        },
        OutgoingCommand::Synchronize(SynchronizeResponse::InSync) => todo!()
    }
}