use engine::{validator_program};
use clap::{Parser};

fn main() {
    let args = Args::parse();

    if let Err(err) = validator_program(args.host, args.port, &args.coordinator_address) {
        println!("Error happened: {}", err)
    }

    println!("Server stopped");
}

#[derive(Parser)]
#[command(about, version)]
struct Args {
    #[arg(long, default_value("0.0.0.0"))]
    host: String,

    #[arg(short, long, default_value("9070"))]
    port: u16,

    #[arg(short, long, default_value("0.0.0.0:9065"))]
    coordinator_address: String
}
