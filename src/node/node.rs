use crate::{
    core::{
        block::Block, consensus::{Chain, ChainError},
        mempool::{Mempool, MempoolError},
        transaction::{SignedTransaction, ValidationError},
        types::{BlockHash, TransactionHash},
    },
    node::peer::{Peer, PeerList},
};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Node {
    pub chain: Chain,
    pub mempool: Mempool,
    pub peer_list: PeerList,
    pub seen_block_hashes: HashSet<BlockHash>,
    pub seen_tx_hashes: HashSet<TransactionHash>,
}

impl Node {
    pub fn new(chain: Chain) -> Self {
        Self {
            chain,
            mempool: Mempool::new(),
            peer_list: PeerList::new(),
            seen_block_hashes: HashSet::new(),
            seen_tx_hashes: HashSet::new(),
        }
    }

    pub fn on_transaction(
        &mut self,
        source_peer: &Peer,
        tx: SignedTransaction,
    ) -> Result<(), NodeError> {
        let tx_hash = tx.tx.hash();

        if self.seen_tx_hashes.contains(&tx_hash) {
            return Err(NodeError::TransactionAlreadySeen);
        }

        self.mempool
            .insert_transaction(&self.chain.state, tx.clone())
            .map_err(NodeError::MempoolError)?;

        self.seen_tx_hashes.insert(tx_hash);

        for _peer in self.peer_list.peers.iter() {
            if _peer != source_peer {
                //TODO: Send to peers via TCP connection
            }
        }

        Ok(())
    }

    pub fn on_block(&mut self, source_peer: &Peer, block: Block) -> Result<(), NodeError> {
        let block_hash = block.hash();

        if self.seen_block_hashes.contains(&block_hash) {
            return Err(NodeError::BlockAlreadySeen);
        }

        self.chain
            .insert_block(block.clone())
            .map_err(NodeError::ChainError)?;

        self.seen_block_hashes.insert(block_hash);

        for tx in &block.transactions {
            let sender = tx.tx.from;
            let nonce = tx.tx.nonce;

            if let Some(account_pool) = self.mempool.by_account.get_mut(&sender) {
                account_pool.remove(&nonce);
                if account_pool.is_empty() {
                    self.mempool.by_account.remove(&sender);
                }
            }
        }

        for _peer in self.peer_list.peers.iter() {
            if _peer != source_peer {
                //TODO: Send to peers via TCP connection
            }
        }

        Ok(())
    }
}

pub enum NodeError {
    TransactionAlreadySeen,
    BlockAlreadySeen,
    ValidationError(ValidationError),
    MempoolError(MempoolError),
    ChainError(ChainError),
}
