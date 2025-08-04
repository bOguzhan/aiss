use aiss::{Client, Error};
use std::net::SocketAddr;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let server_addr: SocketAddr = "127.0.0.1:8080".parse()?;
    let local_addr: SocketAddr = "127.0.0.1:0".parse()?;
    
    let mut client = Client::new("client1".to_string(), local_addr);
    client.register(server_addr).await?;
    
    println!("Client registered successfully");
    
    // Keep the client running
    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }
}