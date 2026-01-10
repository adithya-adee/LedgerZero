use std::{
    io::{Read, Write},
    net::TcpStream,
};

use crate::core::block::Block;
use crate::core::transaction::SignedTransaction;
use serde::{Deserialize, Serialize};

/// Represents the different types of data that can be gossiped in the network
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum GossipData {
    /// A signed transaction to be propagated
    Transaction(SignedTransaction),
    /// A mined block to be propagated
    Block(Block),
}

/// Message structure for gossip protocol
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Gossip {
    pub data: GossipData,
}

impl Gossip {
    /// Create a new gossip message for a transaction
    pub fn new_transaction(tx: SignedTransaction) -> Self {
        Self {
            data: GossipData::Transaction(tx),
        }
    }

    /// Create a new gossip message for a block
    pub fn new_block(block: Block) -> Self {
        Self {
            data: GossipData::Block(block),
        }
    }

    /// Serialize the gossip message to bytes for network transmission
    pub fn to_bytes(&self) -> Result<Vec<u8>, postcard::Error> {
        postcard::to_allocvec(self)
    }

    /// Deserialize a gossip message from bytes received from the network
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, postcard::Error> {
        postcard::from_bytes(bytes)
    }
}

/// Send a gossip message over a TCP stream using length-prefixed framing
/// Format: [4 bytes length (little-endian u32)][payload bytes]
pub fn send_gossip(gossip: &Gossip, stream: &mut TcpStream) -> std::io::Result<()> {
    let payload = gossip
        .to_bytes()
        .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidData, "serialize failed"))?;

    let len = payload.len() as u32;

    stream.write_all(&len.to_le_bytes())?;
    stream.write_all(&payload)?;
    stream.flush()?;

    Ok(())
}

/// Receive a gossip message from a TCP stream using length-prefixed framing
/// Format: [4 bytes length (little-endian u32)][payload bytes]
pub fn receive_gossip(stream: &mut TcpStream) -> std::io::Result<Gossip> {
    let mut len_buf = [0u8; 4];
    stream.read_exact(&mut len_buf)?;

    let len = u32::from_le_bytes(len_buf) as usize;
    let mut payload = vec![0u8; len];

    stream.read_exact(&mut payload)?;

    let gossip = Gossip::from_bytes(&payload)
        .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidData, "bad gossip"))?;

    Ok(gossip)
}
