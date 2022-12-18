use std::{net::TcpStream, io::Write};
use protocol::{external::{ExternalRequest, UserCommand}};
use anyhow::Result;
use serde::Serialize;
use uuid::Uuid;
use clap::Parser;

fn main() {
    let args = Args::parse();

    if let Err(err) = client(&args.destination) {
        println!("Error happened: {}", err);
    }
}

fn client(destination: &str) -> Result<()> {
    let request_id = Uuid::new_v4().to_string();
    let request = &ExternalRequest { 
            request_id, 
            command: UserCommand::CreateRecord { data: "hi there".to_string() } 
        };
    send_bytes(destination, request)?;
    Ok(())
}

fn send_bytes(destination: &str, msg: & impl Serialize) -> Result<()> {
    let mut stream = TcpStream::connect(destination)?;

    let bytes = serde_cbor::to_vec(msg)?;

    stream.write(&bytes)?;
    Ok(())
}

#[derive(Parser)]
#[command(version, about)]
struct Args {
    #[arg(short, long)]
    destination: String
}