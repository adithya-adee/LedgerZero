use crate::core::block::Block;
use crate::core::transaction::SignedTransaction;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum GossipData {
    Transaction(SignedTransaction),
    Block(Block),
}

/// Message structure for gossip protocol
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Gossip {
    pub data: GossipData,
}

impl Gossip {
    pub fn new_transaction(tx: SignedTransaction) -> Self {
        Self {
            data: GossipData::Transaction(tx),
        }
    }

    pub fn new_block(block: Block) -> Self {
        Self {
            data: GossipData::Block(block),
        }
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>, postcard::Error> {
        postcard::to_allocvec(self)
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, postcard::Error> {
        postcard::from_bytes(bytes)
    }
}
