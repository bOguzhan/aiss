use crate::{Registration, Protocol, Error};
use std::net::SocketAddr;
use tokio::net::TcpStream;
use tokio::io::AsyncWriteExt;

pub struct Client {
    id: String,
    local_addr: SocketAddr,
    public_addr: Option<SocketAddr>,
    protocols: Vec<Protocol>,
}

impl Client {
    pub fn new(id: String, local_addr: SocketAddr) -> Self {
        Self {
            id,
            local_addr,
            public_addr: None,
            protocols: vec![Protocol::TCP, Protocol::UDP],
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
        Ok(())
    }
}