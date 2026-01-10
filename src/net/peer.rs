use serde::{Deserialize, Serialize};
use std::net::{IpAddr, TcpStream};
use std::sync::{Arc, RwLock};

use crate::net::gossip::{GossipData, receive_gossip};
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

impl PeerList {
    pub fn new() -> Self {
        Self { peers: Vec::new() }
    }

    pub fn add_peer(&mut self, peer: Peer) {
        if !self.peers.contains(&peer) {
            self.peers.push(peer);
        }
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
