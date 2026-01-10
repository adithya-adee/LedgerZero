use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, RwLock};
use std::thread;

use crate::core::block::Block;
use crate::core::transaction::SignedTransaction;
use crate::net::gossip::{send_gossip, Gossip};
use crate::net::peer::peer_read_loop;
use crate::node::node::Node;

/// Error type for transport layer operations
#[derive(Debug)]
pub enum TransportError {
    IoError(std::io::Error),
}

impl From<std::io::Error> for TransportError {
    fn from(err: std::io::Error) -> Self {
        TransportError::IoError(err)
    }
}

/// Helper: Bind a TCP listener to the given address
fn bind_listener(addr: &str) -> std::io::Result<TcpListener> {
    TcpListener::bind(addr)
}

/// Helper: Connect to a peer at the given address
fn connect_peer(addr: &str) -> std::io::Result<TcpStream> {
    TcpStream::connect(addr)
}

/// Server manages the blockchain node and all peer connections
pub struct Server {
    pub node: Arc<RwLock<Node>>,
    listener: Option<TcpListener>,
    peers: Arc<RwLock<Vec<TcpStream>>>,
}

impl Server {
    /// Create a new server with the given node
    pub fn new(node: Node) -> Self {
        Self {
            node: Arc::new(RwLock::new(node)),
            listener: None,
            peers: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Start the TCP listener on the given address
    /// Spawns a background thread to accept incoming connections
    pub fn start_listener(&mut self, addr: &str) -> std::io::Result<()> {
        let listener = bind_listener(addr)?;
        self.listener = Some(listener.try_clone()?);

        let peers = Arc::clone(&self.peers);
        let node = Arc::clone(&self.node);

        thread::spawn(move || {
            for stream in listener.incoming() {
                match stream {
                    Ok(stream) => {
                        // Clone stream for read loop
                        if let Ok(stream_clone) = stream.try_clone() {
                            peers.write().unwrap().push(stream);

                            let node_clone = Arc::clone(&node);
                            thread::spawn(move || {
                                if let Err(e) = peer_read_loop(stream_clone, node_clone) {
                                    eprintln!("Peer read loop error: {}", e);
                                }
                            });
                        }
                    }
                    Err(e) => {
                        eprintln!("Failed to accept connection: {}", e);
                    }
                }
            }
        });

        Ok(())
    }

    /// Connect to a peer at the given address
    /// Adds the connection to the peer pool and spawns a read loop
    pub fn connect_to_peer(&mut self, addr: &str) -> std::io::Result<()> {
        let stream = connect_peer(addr)?;
        let stream_clone = stream.try_clone()?;

        self.peers.write().unwrap().push(stream);

        let node = Arc::clone(&self.node);
        thread::spawn(move || {
            if let Err(e) = peer_read_loop(stream_clone, node) {
                eprintln!("Peer read loop error: {}", e);
            }
        });

        Ok(())
    }

    /// Broadcast a transaction to all connected peers
    pub fn broadcast_transaction(&self, tx: &SignedTransaction) -> Result<(), TransportError> {
        let gossip = Gossip::new_transaction(tx.clone());
        let mut peers = self.peers.write().unwrap();

        peers.retain_mut(|stream| {
            send_gossip(&gossip, stream).is_ok()
        });

        Ok(())
    }

    /// Broadcast a block to all connected peers
    pub fn broadcast_block(&self, block: &Block) -> Result<(), TransportError> {
        let gossip = Gossip::new_block(block.clone());
        let mut peers = self.peers.write().unwrap();

        peers.retain_mut(|stream| {
            send_gossip(&gossip, stream).is_ok()
        });

        Ok(())
    }

    /// Get the number of active peer connections
    pub fn peer_count(&self) -> usize {
        self.peers.read().unwrap().len()
    }

    /// Get a reference to the node (for reading state, etc.)
    pub fn node(&self) -> Arc<RwLock<Node>> {
        Arc::clone(&self.node)
    }
}