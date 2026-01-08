use crate::core::transaction::SignedTransaction;
use crate::core::types::Address;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct State {
    pub balances: HashMap<Address, u64>,
    pub nonces: HashMap<Address, u64>,
}

/// Applies a validated transaction to the state.
/// Panics if called without prior validation - indicates programmer error.
pub fn apply(state: &mut State, signed_tx: &SignedTransaction) {
    let sender_address = signed_tx.tx.from;
    let receiver_address = signed_tx.tx.to;
    let transfer_amount = signed_tx.tx.amount;
    let fee = signed_tx.tx.fee;

    let sender_balance = state
        .balances
        .get_mut(&sender_address)
        .expect("sender must exist after validation");
    *sender_balance -= transfer_amount + fee;

    let sender_nonce = state
        .nonces
        .get_mut(&sender_address)
        .expect("nonce must exist after validation");
    *sender_nonce += 1;

    *state.balances.entry(receiver_address).or_insert(0) += transfer_amount;
    state.nonces.entry(receiver_address).or_insert(0);
}
