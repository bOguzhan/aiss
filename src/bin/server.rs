// filepath: /Users/mac/Desktop/aiss/aiss/src/bin/server.rs

use aiss::Error;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    println!("Server listening on 127.0.0.1:8080");

    while let Ok((socket, _)) = listener.accept().await {
        tokio::spawn(async move {
            // Handle client connection
            println!("New client connected");
        });
    }

    Ok(())
}