use engine::program;
use clap::Parser;

fn main() {
    let args = Args::parse();

    if let Err(err) = program(args.host, args.port) {
        println!("Error happened: {}", err)
    }

    println!("Server stopped");
}

#[derive(Debug, Parser)]
#[command(about, version)]
struct Args {
    #[arg(long)]
    host: String,

    #[arg(short, long)]
    port: u16,
}