use crate::{
    configuration::{Configuration, ValidatorAddress},
    model::PublicKeyStr,
    request_handlers::handle_request, blockchain::{blockchain::BlockChain, utxo::UnspentOutput}, encryption::{generate_rsa_keypair_custom}, client::{send_bytes}, response_handlers::handle_response,
};
use anyhow::Result;
use protocol::{request::{Request, CommandRequest}, request::Response};
use rsa::RsaPublicKey;
use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream}, sync::{Mutex, Arc}, collections::HashSet,
};

pub fn run_node(host: String, port: u16, root_public_key: &str, remote_validator_opt: Option<&str>) -> Result<()> {
    let listener = TcpListener::bind(format!("{}:{}", host, port))?;
    println!("Validator node is running on {}:{}", host, port);

    let pub_key_str = PublicKeyStr::from_str(root_public_key);
    let pub_key = RsaPublicKey::try_from(&pub_key_str)?;
    let (validator_private_key, validator_public_key) = &generate_rsa_keypair_custom()?;

    let mut configuration = Configuration::new(&host, port, validator_private_key);
    let blockchain = BlockChain::new(validator_public_key, UnspentOutput::initial_utxo(&pub_key_str, 100));
    let blockchain = Arc::new(Mutex::new(blockchain));

    let mut triggered_requests = Vec::new();

    // Register current validator with other validators
    if let Some(remote_validator) = remote_validator_opt {
        let request = CommandRequest::new_on_board_command(&format!("{}:{}", host, port), &validator_public_key.0.0).to_request();
        triggered_requests.push(
            (
                (PublicKeyStr::from_str("not-necessary-here"), ValidatorAddress(remote_validator.to_owned())
            ), request)
        );
    }

    let mut processed_requests = HashSet::<String>::new();

    loop {
        if triggered_requests.is_empty() {
            let (mut stream, addr) = listener.accept()?;
            println!("New connection opened");
    
            let request = receive_and_parse(&mut stream)?;
            let response = if processed_requests.contains(&request.request_id) {
                Response::Error {msg: format!("Request {} already processed", request.request_id)}
            } else {
                processed_requests.insert(request.request_id.to_owned());
                let (response, sub_requests) = handle_request(&request, blockchain.clone(), &mut configuration)?;
                triggered_requests = sub_requests;
                response
            };
            let bytes = serde_cbor::to_vec(&response)?;
            stream.write(&bytes)?;
        } else {
            let mut new_requests = Vec::new();
            for ((_, addr), request) in triggered_requests {
                let blockchain = blockchain.clone();
                println!("Sending triggered request");
                let response = send_bytes(&addr.0, request).unwrap();
                let requests = handle_response(&blockchain, &mut configuration, &response).unwrap_or_else(|err| {println!("{}", err); Vec::new()});
                new_requests.extend(requests);
            }
            triggered_requests = new_requests;
        }
    }
}

/**
 * Reads request from socket and parses it
 */
fn receive_and_parse(stream: &mut TcpStream) -> Result<Request> {
    let mut len: [u8; 8] = [0; 8];
    let mut buf: [u8; 10240] = [0; 10240];
    stream.read(&mut len)?;

    let size_received = usize::from_be_bytes(len);
    stream.read(&mut buf)?;

    let received_msg: Request = serde_cbor::from_slice(&buf[0..size_received])?;
    Ok(received_msg)
}
