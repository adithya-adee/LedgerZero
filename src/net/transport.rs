use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, RwLock};
use std::thread;

use crate::core::block::Block;
use crate::core::transaction::SignedTransaction;
use crate::net::gossip::{Gossip, send_gossip};
use crate::net::peer::peer_read_loop;
use crate::node::node::{Node, NodeError};

pub const HOST_ADDRESS: &str = "127.0.0.1:8080";

/// Error type for transport layer operations
#[derive(Debug)]
pub enum TransportError {
    NodeError(NodeError),
    IoError(std::io::Error),
}

impl From<std::io::Error> for TransportError {
    fn from(err: std::io::Error) -> Self {
        TransportError::IoError(err)
    }
}

impl From<NodeError> for TransportError {
    fn from(err: NodeError) -> Self {
        TransportError::NodeError(err)
    }
}

/// Bind a TCP listener to the given address
/// Returns a TcpListener that can be used to accept incoming connections
pub fn bind_listener(addr: &str) -> std::io::Result<TcpListener> {
    TcpListener::bind(addr)
}

/// Connect to a peer at the given IP address and port
/// Returns a TcpStream if successful
pub fn connect_to_peer(ip: &str, port: u16) -> std::io::Result<TcpStream> {
    TcpStream::connect(format!("{}:{}", ip, port))
}

/// Broadcast a transaction to all connected peers
pub fn broadcast_transaction(
    tx: &SignedTransaction,
    streams: &mut [TcpStream],
) -> Result<(), TransportError> {
    let gossip = Gossip::new_transaction(tx.clone());

    for stream in streams.iter_mut() {
        // Best effort - continue even if one peer fails
        let _ = send_gossip(&gossip, stream);
    }

    Ok(())
}

/// Broadcast a block to all connected peers
pub fn broadcast_block(block: &Block, streams: &mut [TcpStream]) -> Result<(), TransportError> {
    let gossip = Gossip::new_block(block.clone());

    for stream in streams.iter_mut() {
        // Best effort - continue even if one peer fails
        let _ = send_gossip(&gossip, stream);
    }

    Ok(())
}

// TCP Server
pub fn start_listener(node: Arc<RwLock<Node>>) -> std::io::Result<()> {
    let listener = bind_listener(HOST_ADDRESS)?;

    for stream in listener.incoming() {
        let stream = stream?;
        let node_clone = Arc::clone(&node);

        thread::spawn(move || {
            let _ = peer_read_loop(stream, node_clone);
        });
    }

    Ok(())
}
