use engine::run_node;
use clap::{Parser};

fn main() {
    let args = Args::parse();

    if let Err(err) = run_node(args.host, args.port, &args.root_public_key) {
        println!("Error happened: {}", err)
    }

    println!("Server stopped");
}

#[derive(Parser)]
#[command(about, version)]
struct Args {
    #[arg(long, default_value("0.0.0.0"))]
    host: String,

    #[arg(short, long, default_value("9065"))]
    port: u16,

    #[arg(short, long)]
    root_public_key: String,
}
