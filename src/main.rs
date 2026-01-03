use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

pub type Address = [u8; 32];
pub type Signature = [u8; 64];
pub type BlockHash = [u8; 32];

pub const GENESIS_HASH: BlockHash = [0u8; 32];

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Block {
    pub index: u64,
    pub prev_hash: BlockHash,
    pub transactions: Vec<SignedTransaction>,
}

impl Block {
    pub fn hash(&self) -> BlockHash {
        let bytes = serde_json::to_vec(self)
            .expect("block serialization must be deterministic");

        let mut hasher = Sha256::new();
        hasher.update(bytes);
        hasher.finalize().into()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct State {
    pub balances: HashMap<Address, u64>,
    pub nonces: HashMap<Address, u64>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Transaction {
    pub from: Address,
    pub to: Address,
    pub amount: u64,
    pub nonce: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SignedTransaction {
    pub tx: Transaction,
    #[serde(with = "serde_big_array::BigArray")]
    pub signature: Signature,
}

pub fn validate(state: &State, signed_tx: &SignedTransaction) -> Result<(), ValidationError> {
    let sender_address = &signed_tx.tx.from;

    let sender_balance = state
        .balances
        .get(sender_address)
        .ok_or(ValidationError::SenderAccountNotFound)?;

    let current_nonce = state
        .nonces
        .get(sender_address)
        .ok_or(ValidationError::NonceNotFound)?;

    if signed_tx.tx.nonce != current_nonce + 1 {
        return Err(ValidationError::InvalidNonce);
    }

    if *sender_balance < signed_tx.tx.amount {
        return Err(ValidationError::InsufficientBalance);
    }

    if signed_tx.tx.amount == 0 {
        return Err(ValidationError::ZeroAmount);
    }

    Ok(())
}

/// Applies a validated transaction to the state.
/// Panics if called without prior validation - indicates programmer error.
pub fn apply(state: &mut State, signed_tx: &SignedTransaction) {
    let sender_address = signed_tx.tx.from;
    let receiver_address = signed_tx.tx.to;
    let transfer_amount = signed_tx.tx.amount;

    let sender_balance = state
        .balances
        .get_mut(&sender_address)
        .expect("sender must exist after validation");
    *sender_balance -= transfer_amount;

    let sender_nonce = state
        .nonces
        .get_mut(&sender_address)
        .expect("nonce must exist after validation");
    *sender_nonce += 1;

    *state.balances.entry(receiver_address).or_insert(0) += transfer_amount;
    state.nonces.entry(receiver_address).or_insert(0);
}

pub fn genesis_initialization(state: &mut State) -> Result<(), GenesisError> {
    Ok(())
}

fn main() {
    println!("LedgerZero");
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationError {
    SenderAccountNotFound,
    NonceNotFound,
    InvalidNonce,
    InsufficientBalance,
    ZeroAmount,
    InvalidSignature,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GenesisError {
    // Add genesis-specific errors here as needed
}
