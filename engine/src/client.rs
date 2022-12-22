use std::{net::TcpStream, io::Write};

use anyhow::Result;
use protocol::{request::Request, response::Response, external::UserCommand};

pub struct Client {
    destination: String,
}

impl Client {
    pub fn new(destination: &str) -> Self {
        Client { destination: destination.to_string() }
    }

    pub fn ping(&self, msg: &str) -> Result<Response> {
        send_bytes(&self.destination, UserCommand::generate_ping(msg).to_request())
    }

    pub fn register_validator(&self, ip: &str, port: u16) -> Result<Response> {
        todo!()
    }
}

pub fn send_bytes(destination: &str, msg: Request) -> Result<Response> {
    println!("Sending {:?}", msg);
    let mut stream = TcpStream::connect(destination)?;

    let bytes = serde_cbor::to_vec(&msg)?;
    let len: [u8; 8] = bytes.len().to_be_bytes();
    stream.write(&len)?;
    stream.write(&bytes)?;

    let response: Response = serde_cbor::from_reader(&stream)?;

    Ok(response)
}

