use crate::core::block::Block;
use crate::core::transaction::SignedTransaction;
use crate::node::node::{Node, NodeError};
use crate::net::peer::PeerList;

#[derive(Debug)]
pub struct Transport {
    node: Node,
    peer_list: PeerList,
}

impl Transport {
    pub fn new(node: Node, peer_list: PeerList) -> Self {
        Self { node, peer_list }
    }

    pub fn on_transaction(&mut self, tx: SignedTransaction) -> Result<(), TransportError> {
        self.node.on_transaction(tx).map_err(|e| TransportError::NodeError(e))
    }

    pub fn on_block(&mut self, block: Block) -> Result<(), TransportError> {
        self.node.on_block(block).map_err(|e| TransportError::NodeError(e))
    }

    pub fn receive_data(&mut self) -> Result<(), TransportError> {
        // Receive data from peers via TCP Socket connection
        // Parse data
        // Call on_transaction or on_block
    }

    pub fn send_data(&self) -> Result<(), TransportError> {
        // Send data to peers
    }
}

pub enum TransportError {
    NodeError(NodeError),
    IoError(std::io::Error),
}
