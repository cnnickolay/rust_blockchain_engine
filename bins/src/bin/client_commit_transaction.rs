use engine::client::Client;
use anyhow::{Result, anyhow};
use clap::{Parser};
use protocol::{external::{ExternalResponse, UserCommandResponse}, response::Response};

fn main() {
    let args = Args::parse();

    if let Err(err) = client(&args) {
        println!("Error happened: {}", err);
    }
}

fn client(args: &Args) -> Result<()> {
    let client = Client::new(&args.destination);
    let nonce_response = client.generate_nonce(&args.from_address)?;
    if let Response::External(ExternalResponse::Success(UserCommandResponse::GenerateNonceResponse{ref nonce})) = nonce_response {
        println!("Received {:?}", client.send_transaction(nonce, &args.from_address, &args.to_address, args.amount, &args.private_key)?);
        Ok(())
    } else {
        Err(anyhow!("Failed to retrieve nonce"))
    }
}

#[derive(Parser)]
#[command(version, about)]
struct Args {
    #[arg(short, long, default_value("0.0.0.0:9065"))]
    destination: String,

    #[arg(short, long)]
    from_address: String,

    #[arg(short, long)]
    private_key: String,

    #[arg(short, long)]
    to_address: String,

    #[arg(short, long)]
    amount: u64,
}
