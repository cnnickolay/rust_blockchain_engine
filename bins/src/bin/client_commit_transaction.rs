use engine::client::Client;
use anyhow::Result;
use clap::Parser;


fn main() {
    let args = Args::parse();

    if let Err(err) = client(&args) {
        println!("Error happened: {}", err);
    }
}

fn client(args: &Args) -> Result<()> {
    let client = Client::new(&args.destination);
    println!("Received {:?}", client.send_transaction(&args.from_address, &args.to_address, args.amount)?);
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
