use engine::client::Client;
use anyhow::Result;
use clap::Parser;
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
    let balanced_transaction_response = client.balance_transaction(&args.from_address, &args.to_address, args.amount)?;
    println!("{}", serde_json::to_string_pretty(&balanced_transaction_response)?);
    Ok(())
}

#[derive(Parser)]
#[command(version, about)]
struct Args {
    #[arg(short, long, default_value("0.0.0.0:9065"))]
    destination: String,

    #[arg(short, long)]
    from_address: String,

    #[arg(short, long)]
    to_address: String,

    #[arg(short, long)]
    amount: u64,
}
