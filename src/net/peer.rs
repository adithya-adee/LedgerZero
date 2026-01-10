use serde::{Deserialize, Serialize};
use std::net::{IpAddr, SocketAddr, TcpStream};
use std::sync::{Arc, RwLock};

use crate::net::gossip::{receive_gossip, GossipData};
use crate::node::node::Node;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PeerList {
    pub peers: Vec<Peer>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Peer {
    pub ip: IpAddr,
    pub port: u16,
}

impl Peer {
    /// Create a new peer from IP and port
    pub fn new(ip: IpAddr, port: u16) -> Self {
        Self { ip, port }
    }

    /// Get the socket address for this peer
    pub fn addr(&self) -> SocketAddr {
        SocketAddr::new(self.ip, self.port)
    }

    /// Get the address as a string (for connecting)
    pub fn addr_string(&self) -> String {
        format!("{}:{}", self.ip, self.port)
    }
}

impl PeerList {
    pub fn new() -> Self {
        Self { peers: Vec::new() }
    }

    pub fn add_peer(&mut self, peer: Peer) {
        if !self.peers.contains(&peer) {
            self.peers.push(peer);
        }
    }

    /// Find a peer by address
    pub fn find_peer(&self, ip: IpAddr, port: u16) -> Option<&Peer> {
        self.peers.iter().find(|p| p.ip == ip && p.port == port)
    }
}

/// Main loop for reading gossip messages from a peer connection
/// Runs indefinitely until the connection is closed or an error occurs
pub fn peer_read_loop(mut stream: TcpStream, node: Arc<RwLock<Node>>) -> std::io::Result<()> {
    loop {
        let gossip = receive_gossip(&mut stream)?;

        match gossip.data {
            GossipData::Transaction(tx) => {
                let _ = node.write().unwrap().on_transaction(tx);
            }
            GossipData::Block(block) => {
                let _ = node.write().unwrap().on_block(block);
            }
        }
    }
}
