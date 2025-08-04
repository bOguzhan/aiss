use crate::Error;
use std::net::SocketAddr;
use tokio::net::{TcpStream, TcpListener, UdpSocket};

pub async fn udp_hole_punch(peer_addr: SocketAddr) -> Result<UdpSocket, Error> {
    let socket = UdpSocket::bind("0.0.0.0:0").await?;
    
    socket.send_to(b"probe", peer_addr).await?;
    
    let mut buf = [0u8; 1024];
    loop {
        match socket.recv_from(&mut buf).await {
            Ok((_, addr)) if addr == peer_addr => {
                return Ok(socket);
            }
            _ => continue,
        }
    }
}

pub async fn tcp_simultaneous_open(peer_addr: SocketAddr) -> Result<TcpStream, Error> {
    let local_addr = "0.0.0.0:0";
    let listener = TcpListener::bind(local_addr).await?;
    
    tokio::select! {
        result = TcpStream::connect(peer_addr) => {
            result.map_err(Error::from)
        }
        result = listener.accept() => {
            result.map(|(stream, _)| stream).map_err(Error::from)
        }
    }
}