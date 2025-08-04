// filepath: /Users/mac/Desktop/aiss/aiss/src/bin/server.rs

use aiss::{Registration, PeerInfo, Error};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use clap::Parser;
use tokio_rustls::TlsAcceptor;
use rustls::{ServerConfig, Certificate, PrivateKey};
use std::fs::File;
use std::io::BufReader;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value = "8081")]
    port: u16,

    #[arg(short, long)]
    cert_path: String,

    #[arg(short, long)]
    key_path: String,
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

async fn load_tls_config(cert_path: &str, key_path: &str) -> Result<ServerConfig, Error> {
    let cert_file = File::open(cert_path)?;
    let key_file = File::open(key_path)?;
    
    let cert_chain = rustls_pemfile::certs(&mut BufReader::new(cert_file))?
        .into_iter()
        .map(Certificate)
        .collect();
    
    let key = rustls_pemfile::private_key(&mut BufReader::new(key_file))?
        .ok_or("no private key found")?;

    let config = ServerConfig::builder()
        .with_safe_defaults()
        .with_no_client_auth()
        .with_single_cert(cert_chain, PrivateKey(key))?;
    
    Ok(config)
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let args = Args::parse();
    let addr = format!("0.0.0.0:{}", args.port);
    
    // Load TLS configuration
    let tls_config = load_tls_config(&args.cert_path, &args.key_path).await?;
    let acceptor = TlsAcceptor::from(Arc::new(tls_config));
    
    let listener = TcpListener::bind(&addr).await?;
    println!("Secure server listening on {}", addr);

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