use std::collections::HashMap;
use std::net::SocketAddr;
use tokio::net::{TcpStream, TcpListener, UdpSocket};
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Protocol {
    TCP,
    UDP,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NatType {
    FullCone,
    RestrictedCone,
    PortRestrictedCone,
    Symmetric,
    CGNat,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Registration {
    client_id: String,
    local_addr: SocketAddr,
    public_addr: Option<SocketAddr>,
    protocols: Vec<Protocol>,
}

pub type Error = Box<dyn std::error::Error + Send + Sync>;