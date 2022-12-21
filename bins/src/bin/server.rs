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
    #[arg(long, default_value("0.0.0.0"))]
    host: String,

    #[arg(short, long, default_value("9065"))]
    port: u16,
}