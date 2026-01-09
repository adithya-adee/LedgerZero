use crate::core::transaction::SignedTransaction;
use crate::core::types::{Address, BlockHash};
use crate::crypto::hash::sha256;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Block {
    pub prev_hash: BlockHash,
    pub producer: Address,
    pub nonce: u64,
    pub transactions: Vec<SignedTransaction>,
}

impl Block {
    pub fn hash(&self) -> BlockHash {
        sha256(self)
    }

    pub fn work(&self) -> u128 {
        1
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BlockMeta {
    pub height: u64,
    pub parent: BlockHash,
    pub total_work: u128,
}
