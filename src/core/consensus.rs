use crate::core::pow::valid_pow;
use crate::core::transaction::validate;
use crate::core::{state::apply, transaction::ValidationError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::core::{
    block::{Block, BlockMeta},
    state::State,
    types::{BlockHash, GENESIS_HASH, ZERO_ADDRESS},
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Chain {
    pub blocks: HashMap<BlockHash, Block>,
    pub meta: HashMap<BlockHash, BlockMeta>,
    pub tip: BlockHash,
    pub genesis_state: State,
    pub state: State,
}

impl Chain {
    pub fn new(genesis_state: State, genesis_block: Block) -> Self {
        let genesis_hash = genesis_block.hash();

        let mut blocks = HashMap::new();
        let mut meta = HashMap::new();

        blocks.insert(genesis_hash, genesis_block);
        meta.insert(
            genesis_hash,
            BlockMeta {
                height: 0,
                parent: GENESIS_HASH,
                total_work: 0,
            },
        );

        Self {
            blocks,
            meta,
            tip: genesis_hash,
            genesis_state: genesis_state.clone(),
            state: genesis_state,
        }
    }

    /// Create a chain from a list of blocks loaded from storage
    pub fn from_blocks(genesis_state: State, blocks: Vec<Block>) -> Result<Self, ChainError> {
        // Create genesis block if no blocks provided
        let genesis_block = if blocks.is_empty() {
            Block {
                prev_hash: GENESIS_HASH,
                producer: ZERO_ADDRESS,
                nonce: 0,
                transactions: vec![],
            }
        } else {
            blocks[0].clone()
        };

        let mut chain = Chain::new(genesis_state, genesis_block);

        // Skip genesis block and replay all others
        for block in blocks.into_iter().skip(1) {
            chain.insert_block(block)?;
        }

        Ok(chain)
    }

    pub fn insert_block(&mut self, block: Block) -> Result<(), ChainError> {
        let hash = block.hash();

        // Reject duplicate blocks
        if self.blocks.contains_key(&hash) {
            return Err(ChainError::DuplicateBlock);
        }

        // All blocks must have valid PoW (no exceptions)
        if !valid_pow(&block) {
            return Err(ChainError::InvalidPoW);
        }

        let parent_hash = block.prev_hash;

        // Parent must exist in the chain
        if !self.blocks.contains_key(&parent_hash) {
            return Err(ChainError::UnknownParent);
        }

        // Calculate height and total work based on parent
        let parent_meta = &self.meta[&parent_hash];
        let height = parent_meta.height + 1;
        let total_work = parent_meta.total_work + block.work();

        self.blocks.insert(hash, block);
        self.meta.insert(
            hash,
            BlockMeta {
                height,
                parent: parent_hash,
                total_work,
            },
        );

        if total_work > self.meta[&self.tip].total_work {
            self.reorg_to(hash)?;
        }

        Ok(())
    }

    pub fn reorg_to(&mut self, new_tip: BlockHash) -> Result<(), ChainError> {
        // reset state
        self.state = self.genesis_state.clone();

        let mut path: Vec<BlockHash> = Vec::new();
        let mut cursor = new_tip;

        while cursor != GENESIS_HASH {
            path.push(cursor);
            cursor = self.meta[&cursor].parent;
        }

        path.reverse();

        for hash in path {
            let block = &self.blocks[&hash];

            for tx in &block.transactions {
                validate(&self.state, tx).map_err(ChainError::TransactionValidationFailed)?;
                apply(&mut self.state, tx);
            }

            let fees: u64 = block.transactions.iter().map(|t| t.tx.fee).sum();
            *self.state.balances.entry(block.producer).or_insert(0) += fees;
        }

        self.tip = new_tip;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChainError {
    InvalidPreviousHash,
    InvalidPoW,
    UnknownParent,
    DuplicateBlock,
    TransactionValidationFailed(ValidationError),
}
