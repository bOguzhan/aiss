use crate::{Registration, Protocol, Error, PeerInfo};
use std::net::SocketAddr;
use tokio::net::{TcpStream, TcpListener};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use crate::hole_punch::{udp_hole_punch, tcp_simultaneous_open};
use std::str::FromStr;
use std::collections::HashMap;

#[derive(Debug)]
pub struct Client {
    pub id: String,
    pub local_addr: SocketAddr,
    pub public_addr: Option<SocketAddr>,
    pub protocols: Vec<Protocol>,
    forward_addr: Option<SocketAddr>,
    target_addr: Option<String>,
    peer_connections: HashMap<String, TcpStream>,
}

impl Client {
    pub fn new(
        id: String, 
        local_addr: SocketAddr,
        forward: Option<String>,
        target: Option<String>,
    ) -> Self {
        Self {
            id,
            local_addr,
            public_addr: None,
            protocols: vec![Protocol::TCP, Protocol::UDP],
            forward_addr: forward.map(|f| SocketAddr::from_str(&f).unwrap()),
            target_addr: target,
            peer_connections: HashMap::new(),
        }
    }

    pub async fn register(&mut self, server: SocketAddr) -> Result<(), Error> {
        let mut stream = TcpStream::connect(server).await?;
        
        // In a real implementation, we would detect the public address using STUN
        self.public_addr = Some(server);
        
        let registration = Registration {
            client_id: self.id.clone(),
            local_addr: self.local_addr,
            public_addr: self.public_addr,
            protocols: self.protocols.clone(),
        };
        
        stream.write_all(&serde_json::to_vec(&registration)?).await?;
        self.handle_server_response(stream).await?;
        Ok(())
    }

    async fn connect_to_peer(&self, peer: PeerInfo) -> Result<(), Error> {
        println!("Attempting to connect to peer: {}", peer.id);
        
        // Try UDP first
        if let Ok(udp_socket) = udp_hole_punch(peer.public_addr).await {
            println!("UDP connection established with {}", peer.id);
            return Ok(());
        }
        
        // Fallback to TCP
        if let Ok(tcp_stream) = tcp_simultaneous_open(peer.public_addr).await {
            println!("TCP connection established with {}", peer.id);
            return Ok(());
        }
        
        Err("Failed to connect to peer".into())
    }

    async fn handle_server_response(&self, mut stream: TcpStream) -> Result<(), Error> {
        let mut buffer = Vec::new();
        stream.read_to_end(&mut buffer).await?;
        
        let peer_info: PeerInfo = serde_json::from_slice(&buffer)?;
        self.connect_to_peer(peer_info).await?;
        
        Ok(())
    }

    pub async fn start_forwarding(&mut self) -> Result<(), Error> {
        if let Some(forward_addr) = self.forward_addr {
            let listener = TcpListener::bind(forward_addr).await?;
            println!("Listening for connections on {}", forward_addr);

            while let Ok((inbound, _)) = listener.accept().await {
                let target_addr = self.target_addr.as_ref()
                    .ok_or("No target address specified")?;
                let peer_stream = self.peer_connections.get(target_addr)
                    .ok_or("No connection to target peer")?;
                
                // Create a new connection for each inbound request
                let outbound = TcpStream::connect(peer_stream.peer_addr()?).await?;
                
                tokio::spawn(handle_tunnel(inbound, outbound));
            }
        }
        Ok(())
    }
}

async fn handle_tunnel(mut inbound: TcpStream, mut outbound: TcpStream) {
    let (mut ri, mut wi) = inbound.split();
    let (mut ro, mut wo) = outbound.split();

    let client_to_server = async {
        let result = tokio::io::copy(&mut ri, &mut wo).await;
        wo.shutdown().await?;
        result
    };

    let server_to_client = async {
        let result = tokio::io::copy(&mut ro, &mut wi).await;
        wi.shutdown().await?;
        result
    };

    tokio::select! {
        _ = client_to_server => {},
        _ = server_to_client => {},
    }
}