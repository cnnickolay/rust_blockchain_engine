use anyhow::Result;
use clap::{Parser, ValueEnum};
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
    // let request = match args.command {
    //     ArgsCommand::CreateRecord => UserCommand::CreateRecord { data: args.value.clone() }.to_request(),
    //     ArgsCommand::Ping => UserCommand::generate_ping("ping").to_request()
    // };

    let client = Client::new(&args.destination);
    println!("Received {:?}", client.ping("123")?);
    println!("Received {:?}", client.ping("iiii")?);
    println!("Received {:?}", client.ping("4564095")?);
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
    value: String,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum ArgsCommand {
    CreateRecord,
    Ping,
}
