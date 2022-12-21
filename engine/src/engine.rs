use std::{net::{TcpListener, TcpStream}, io::{Write, Read}};
use anyhow::Result;
use protocol::{request::Request, response::Response};
use crate::{model::State, request_handler::RequestHandler};

pub fn run(host: String, port: u16) -> Result<()> {
    let listener = TcpListener::bind(format!("{}:{}", host, port))?;
    println!("Server is running on {}:{}", host, port);

    let state = State::new();

    loop {
        let (mut stream, addr) = listener.accept()?;
        println!("New connection opened");
    
        let mut len: [u8; 8] = [0; 8];
        let mut buf: [u8; 1024] = [0; 1024];
        stream.read(&mut len)?;
        let size_received = usize::from_be_bytes(len);
        println!("Message size received: {}", size_received);
        stream.read(&mut buf)?;
        let bytes_to_parse = &buf[0..size_received];
        println!("Received bytes: {:?}", bytes_to_parse);
        let received_msg: Request = serde_cbor::from_slice(&bytes_to_parse)?;
        println!("Received: {:?}", received_msg);

        handle_request(&received_msg, &mut stream)?;

        println!("Blockchain elements: {:?}", state.block_chain);
    }
}

pub fn handle_request(request: &Request, stream: &mut TcpStream) -> Result<()> {
    println!("Received request: {:?}", request);
    let response = match request {
        Request::Internal(req) => Response::Internal(req.handle_request()),
        Request::External(req) => Response::External(req.handle_request()),
    };

    let bytes = serde_cbor::to_vec(&response)?;
    println!("Sending response {} bytes", bytes.len());
    stream.write(&bytes)?;
    Ok(())
}