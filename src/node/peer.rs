use serde::{Deserialize, Serialize};
use std::net::IpAddr;

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
}
