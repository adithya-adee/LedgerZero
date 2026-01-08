use crate::{
    core::transaction::SignedTransaction,
    core::types::{Address, BlockHash},
};
use postcard;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Block {
    pub prev_hash: BlockHash,
    pub producer: Address,
    pub nonce: u64,
    pub transactions: Vec<SignedTransaction>,
}

impl Block {
    pub fn hash(&self) -> BlockHash {
        let bytes = postcard::to_allocvec(self).expect("block serialization must be deterministic");

        let mut hasher = Sha256::new();
        hasher.update(bytes);
        hasher.finalize().into()
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
