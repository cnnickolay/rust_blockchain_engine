use std::{net::{TcpStream, TcpListener}, io::Read, str::from_utf8};
use anyhow::Result;
use super::model::Block;
use protocol::{internal::InternalRequest, external::{UserCommand, ExternalRequest}};

pub fn run(host: String, port: u16) -> Result<()> {
    let listener = TcpListener::bind(format!("{}:{}", host, port))?;
    println!("Server is running on {}:{}", host, port);

    let mut state = State::new();

    loop {
        let (mut stream, addr) = listener.accept()?;
        println!("New connection opened");
    
        let received_msg: ExternalRequest = serde_cbor::from_reader(stream)?;

        println!("Received: {:?}", received_msg);

        state.add_block(received_msg);
        println!("Blockchain elements: {:?}", state.block_chain);
    }
}

/**
 * Holds an internal state of the current node
 */
#[derive(Debug)]
 struct State {
    block_chain: Vec<Block>
}

impl State {
    fn new() -> State {
        State {
            block_chain: Vec::new()
        }
    }

    fn add_block(&mut self, request: ExternalRequest) {
        self.block_chain.push(Block::new(request.command))
    }
}