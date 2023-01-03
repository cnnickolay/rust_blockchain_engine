use crate::{
    client::Client,
    configuration::{Configuration, NodeType},
    model::PublicKeyStr,
    request_handler::RequestHandler, blockchain::{blockchain::BlockChain, utxo::UnspentOutput}, encryption::{generate_rsa_key_pair, generate_rsa_keypair_custom},
};
use anyhow::Result;
use protocol::{request::Request, response::Response, internal::InternalResponse, external::ExternalResponse};
use rsa::RsaPublicKey;
use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
};

pub fn run_coordinator_node(host: String, port: u16, root_public_key: &str) -> Result<()> {
    let listener = TcpListener::bind(format!("{}:{}", host, port))?;
    println!("Coordinator node is running on {}:{}", host, port);

    let pub_key_str = PublicKeyStr::from_str(root_public_key);
    let pub_key = RsaPublicKey::try_from(&pub_key_str)?;
    let (validator_private_key, validator_public_key) = &generate_rsa_keypair_custom()?;

    let mut configuration = Configuration::new(&host, port, NodeType::new_coordinator(), validator_private_key);
    let mut blockchain = BlockChain::new(validator_public_key, UnspentOutput::new(&pub_key_str, 100));

    loop {
        let (mut stream, addr) = listener.accept()?;
        println!("New connection opened");

        let request = receive_and_parse(&mut stream)?;
        handle_request(&request, &mut stream, &mut blockchain, &mut configuration)?
    }
}

pub fn run_validator_node(host: String, port: u16, coordinator_ip: &str) -> Result<()> {
    let listener = TcpListener::bind(format!("{}:{}", host, port))?;
    println!("Validator node is running on {}:{}", host, port);

    let client = Client::new(coordinator_ip);
    client.register_validator(&host, port)?;

    loop {
        let (mut stream, addr) = listener.accept()?;
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

/**
 * Handles the request and sends response to the socket
 */
pub fn handle_request(
    request: &Request,
    stream: &mut TcpStream,
    blockchain: &mut BlockChain,
    configuration: &mut Configuration,
) -> Result<()> {
    println!("Received request: {:?}", request);
    let response = match request {
        Request::Internal(req) => {
            match req.handle_request(blockchain, configuration) {
                Ok(result) => Response::Internal(result),
                Err(err) => Response::Internal(InternalResponse::Error { msg: format!("{:?}", err) }),
            }
            
        }
        Request::External(req) => {
            match req.handle_request(blockchain, configuration) {
                Ok(result) => Response::External(result),
                Err(err) => Response::External(ExternalResponse::Error { msg: format!("{:?}", err) }),
            }
        }
    };

    let bytes = serde_cbor::to_vec(&response)?;
    stream.write(&bytes)?;
    Ok(())
}
