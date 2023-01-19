use engine::run_node;
use clap::Parser;
use log::error;


#[tokio::main]
async fn main() {
    env_logger::init();
    let args = Args::parse();

    if let Err(err) = run_node(args.host, args.port, args.remote_validator.as_deref(), &args.private_key, &args.public_key).await {
        error!("Error happened: {}", err)
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

    #[arg(long, default_value(None))]
    remote_validator: Option<String>,

    #[arg(long)]
    private_key: String,

    #[arg(long)]
    public_key: String
}
