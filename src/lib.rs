pub mod client;
pub mod hole_punch;

use std::net::SocketAddr;
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
pub struct PeerInfo {
    pub id: String,
    pub public_addr: SocketAddr,
    pub local_addr: SocketAddr,
    pub protocols: Vec<Protocol>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Registration {
    pub client_id: String,
    pub local_addr: SocketAddr,
    pub public_addr: Option<SocketAddr>,
    pub protocols: Vec<Protocol>,
}

pub type Error = Box<dyn std::error::Error + Send + Sync>;