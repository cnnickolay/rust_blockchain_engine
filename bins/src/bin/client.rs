use std::{net::TcpStream, io::{Write}};
use protocol::{external::{ExternalRequest, UserCommand}, request::Request, response::Response};
use anyhow::Result;

use uuid::Uuid;
use clap::{Parser, ValueEnum};

fn main() {
    let args = Args::parse();

    if let Err(err) = client(&args) {
        println!("Error happened: {}", err);
    }
}

fn client(args: &Args) -> Result<()> {
    let request_id = Uuid::new_v4().to_string();
    let user_command = match args.command { 
        ArgsCommand::CreateRecord => UserCommand::CreateRecord { data: args.value.clone() },
        ArgsCommand::Ping => UserCommand::PingCommand { msg: "This is my PING PING PING PING".to_string() }
    };

    let request = ExternalRequest { 
        request_id, 
        command: user_command, 
    };
    send_bytes(&args.destination, Request::External(request))?;
    Ok(())
}

fn send_bytes(destination: &str, msg: Request) -> Result<()> {
    let mut stream = TcpStream::connect(destination)?;

    let bytes = serde_cbor::to_vec(&msg)?;
    println!("Sending bytes: {:?}", bytes);
    let len: [u8; 8] = bytes.len().to_be_bytes();
    println!("Sending total {} bytes", bytes.len());
    stream.write(&len)?;
    stream.write(&bytes)?;

    let response: Response = serde_cbor::from_reader(&stream)?;
    println!("Response: {:?}", response);

    Ok(())
}

#[derive(Parser)]
#[command(version, about)]
struct Args {
    #[arg(short, long, default_value("0.0.0.0:9065"))]
    destination: String,
    
    #[arg(long, default_value("ping"))]
    #[clap(value_enum)]
    command: ArgsCommand,

    #[arg(long, default_value(""))]
    value: String
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum ArgsCommand {
    CreateRecord,
    Ping
}