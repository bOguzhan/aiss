// filepath: /Users/mac/Desktop/aiss/aiss/src/bin/server.rs

use aiss::{Registration, PeerInfo, Error};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use clap::Parser;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value = "8080")]
    port: u16,
}

#[derive(Clone)]
struct Server {
    clients: Arc<Mutex<HashMap<String, PeerInfo>>>,
}

impl Server {
    fn new() -> Self {
        Self {
            clients: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    async fn handle_client(&self, mut socket: TcpStream) -> Result<(), Error> {
        let mut buffer = Vec::new();
        socket.read_to_end(&mut buffer).await?;
        
        let registration: Registration = serde_json::from_slice(&buffer)?;
        
        // Store client info
        let peer_info = PeerInfo {
            id: registration.client_id.clone(),
            public_addr: registration.public_addr.unwrap(),
            local_addr: registration.local_addr,
            protocols: registration.protocols,
        };

        let mut clients = self.clients.lock().await;
        clients.insert(registration.client_id, peer_info.clone());

        // Notify other clients about the new peer
        for (_, other_peer) in clients.iter() {
            if other_peer.id != peer_info.id {
                socket.write_all(&serde_json::to_vec(&other_peer)?).await?;
            }
        }

        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let args = Args::parse();
    let server = Server::new();
    let addr = format!("0.0.0.0:{}", args.port);
    let listener = TcpListener::bind(&addr).await?;
    println!("Server listening on {}", addr);

    while let Ok((socket, addr)) = listener.accept().await {
        println!("New connection from: {}", addr);
        let server = server.clone();
        tokio::spawn(async move {
            if let Err(e) = server.handle_client(socket).await {
                eprintln!("Error handling client: {}", e);
            }
        });
    }

    Ok(())
}