use aiss::client::Client;
use aiss::Error;
use std::net::SocketAddr;
use clap::Parser;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    id: String,

    #[arg(short, long)]
    forward: Option<String>,

    #[arg(short, long)]
    target: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let args = Args::parse();
    let server_addr: SocketAddr = "127.0.0.1:8080".parse()?;
    let local_addr: SocketAddr = "127.0.0.1:0".parse()?;
    
    let mut client = Client::new(
        args.id,
        local_addr,
        args.forward,
        args.target
    );
    client.register(server_addr).await?;
    
    println!("Client registered successfully");
    client.start_forwarding().await?;
    
    tokio::signal::ctrl_c().await?;
    Ok(())
}