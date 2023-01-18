use engine::client::Client;
use anyhow::Result;
use clap::{Parser};
use log::error;

fn main() {
    env_logger::init();
    let args = Args::parse();

    if let Err(err) = client(&args) {
        error!("Error happened: {}", err);
    }
}

fn client(args: &Args) -> Result<()> {
    let client = Client::new(&args.destination);
    println!("Received {:?}", client.print_balances()?);
    Ok(())
}

#[derive(Parser)]
#[command(version, about)]
struct Args {
    #[arg(short, long, default_value("0.0.0.0:9065"))]
    destination: String,
}
