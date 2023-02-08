use anyhow::Result;
use clap::Parser;
use engine::client::Client;
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
    println!(
        "{:?}",
        client.commit_transaction(&args.cbor, &args.private_key)?
    );
    Ok(())
}

#[derive(Parser)]
#[command(version, about)]
struct Args {
    #[arg(short, long, default_value("0.0.0.0:9065"))]
    destination: String,

    #[arg(short, long)]
    cbor: String,

    #[arg(short, long)]
    private_key: String,
}
