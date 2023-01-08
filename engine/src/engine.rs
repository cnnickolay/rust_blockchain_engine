use crate::{
    configuration::{Configuration, ValidatorAddress},
    model::PublicKeyStr,
    request_handler::RequestHandler, blockchain::{blockchain::BlockChain, utxo::UnspentOutput}, encryption::{generate_rsa_keypair_custom}, client::Client,
};
use anyhow::Result;
use protocol::{request::Request, response::Response, internal::InternalResponse, external::ExternalResponse};
use rsa::RsaPublicKey;
use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream}, sync::{Mutex, Arc},
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

    if let Some(remote_validator) = remote_validator_opt {
        send_on_boarding_request(&mut configuration, &host, port, remote_validator, validator_public_key)?;
    }

    loop {
        let (mut stream, addr) = listener.accept()?;
        println!("New connection opened");

        let request = receive_and_parse(&mut stream)?;
        handle_request(&request, &mut stream, blockchain.clone(), &mut configuration)?
    }
}

/**
 * Sends onboarding request to another validator to build a network of validator nodes
 */
pub fn send_on_boarding_request(configuration: &mut Configuration, ip: &str, port: u16, remote_validator_address: &str, public_key: &PublicKeyStr) -> Result<()> {
    let client = Client::new(remote_validator_address);
    let response = client.register_validator(&format!("{}:{}", ip, port), public_key, false)?;
    let new_validators: Vec<_> = response.iter().map(|v| (PublicKeyStr::from_str(&v.public_key), ValidatorAddress(v.address.to_owned()))).collect();

    // extending the list of known validators
    configuration.add_validators(&new_validators);

    println!("Validators added: {:?}", configuration.validators.iter().map(|validator| &validator.1).collect::<Vec<&ValidatorAddress>>());

    Ok(())
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
    blockchain: Arc<Mutex<BlockChain>>,
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
