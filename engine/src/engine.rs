use std::{net::{TcpListener, TcpStream}, io::{Write, Read}};
use anyhow::Result;
use protocol::{request::Request, response::Response};
use crate::{request_handler::RequestHandler, configuration::{Configuration, NodeType}, client::Client};

pub fn run_coordinator_node(host: String, port: u16) -> Result<()> {
    let listener = TcpListener::bind(format!("{}:{}", host, port))?;
    println!("Coordinator node is running on {}:{}", host, port);

    let mut configuration = Configuration::new(&host, port, NodeType::new_coordinator());

    loop {
        let (mut stream, addr) = listener.accept()?;
        println!("New connection opened");
    
        let request = receive_and_parse(&mut stream)?;
        handle_request(&request, &mut stream, &mut configuration)?;
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
    let mut buf: [u8; 1024] = [0; 1024];
    stream.read(&mut len)?;

    let size_received = usize::from_be_bytes(len);
    stream.read(&mut buf)?;
    
    let received_msg: Request = serde_cbor::from_slice(&buf[0..size_received])?;
    Ok(received_msg)
}

/**
 * Handles the request and sends response to the socket
 */
pub fn handle_request(request: &Request, stream: &mut TcpStream, configuration: &mut Configuration) -> Result<()> {
    println!("Received request: {:?}", request);
    let response = match request {
        Request::Internal(req) => Response::Internal(req.handle_request(configuration)?),
        Request::External(req) => Response::External(req.handle_request(configuration)?),
    };

    let bytes = serde_cbor::to_vec(&response)?;
    stream.write(&bytes)?;
    Ok(())
}