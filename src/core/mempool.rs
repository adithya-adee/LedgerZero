use crate::core::block::Block;
use crate::core::consensus::Chain;
use crate::core::state::apply;
use crate::core::transaction::SignedTransaction;
use crate::core::transaction::ValidationError;
use crate::core::transaction::validate;
use crate::core::types::Address;
use std::collections::BTreeMap;
use std::collections::HashMap;

pub struct Mempool {
    /// Maps sender addresses to their pending transactions, ordered by nonce
    pub by_account: HashMap<Address, BTreeMap<u64, SignedTransaction>>,
}

impl Mempool {
    pub fn new() -> Self {
        Self {
            by_account: HashMap::new(),
        }
    }

    pub fn insert_transaction(
        &mut self,
        chain: &Chain,
        tx: SignedTransaction,
    ) -> Result<(), MempoolError> {
        validate(&chain.state, &tx).map_err(MempoolError::InvalidTransaction)?;

        let sender = tx.tx.from;
        let tx_nonce = tx.tx.nonce;

        let state_nonce = *chain.state.nonces.get(&sender).unwrap_or(&0);

        if tx_nonce <= state_nonce {
            return Err(MempoolError::NonceTooLow);
        }

        let account_pool = self.by_account.entry(sender).or_default();

        if !account_pool.is_empty() {
            let last_nonce = *account_pool.keys().last().unwrap();

            if tx_nonce != last_nonce + 1 {
                return Err(MempoolError::NonceGap);
            }
        } else {
            if tx_nonce != state_nonce + 1 {
                return Err(MempoolError::NonceGap);
            }
        }

        if account_pool.contains_key(&tx_nonce) {
            return Err(MempoolError::DuplicateTransaction);
        }

        // Calculate pending balance usage from already-queued transactions
        let pending_usage: u64 = account_pool
            .values()
            .map(|pending_tx| pending_tx.tx.amount + pending_tx.tx.fee)
            .sum();

        // Check if sender has enough balance for pending txs + new tx
        let sender_balance = *chain.state.balances.get(&sender).unwrap_or(&0);
        let new_tx_cost = tx.tx.amount + tx.tx.fee;
        let total_required = pending_usage
            .checked_add(new_tx_cost)
            .ok_or(MempoolError::InsufficientBalance)?;

        if sender_balance < total_required {
            return Err(MempoolError::InsufficientBalance);
        }

        account_pool.insert(tx_nonce, tx);

        Ok(())
    }
}

/// Assembles a new block from pending mempool transactions.
/// Iterates accounts in sorted order for deterministic block assembly.
pub fn assemble_block(chain: &Chain, mempool: &Mempool, producer: Address) -> Block {
    let mut temp_state = chain.state.clone();
    let mut transactions: Vec<SignedTransaction> = Vec::new();

    // Collect and sort addresses for deterministic iteration
    let mut sorted_addresses: Vec<&Address> = mempool.by_account.keys().collect();
    sorted_addresses.sort();

    // Iterate over accounts in sorted order
    for addr in sorted_addresses {
        let map = &mempool.by_account[addr];
        for (_nonce, tx) in map {
            if validate(&temp_state, tx).is_ok() {
                apply(&mut temp_state, tx);
                transactions.push(tx.clone());
            } else {
                break;
            }
        }
    }

    Block {
        prev_hash: chain.tip,
        producer,
        nonce: 0,
        transactions,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MempoolError {
    InvalidTransaction(ValidationError),
    NonceTooLow,
    NonceGap,
    DuplicateTransaction,
    InsufficientBalance,
}
