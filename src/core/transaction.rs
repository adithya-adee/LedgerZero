use crate::core::state::State;
use crate::core::types::{Address, PublicKeyBytes, Signature, TransactionHash};
use crate::crypto::hash::sha256;
use crate::crypto::signature::{address_from_pubkey, verify_signature};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Transaction {
    pub from: Address,
    pub to: Address,
    pub amount: u64,
    pub fee: u64,
    pub nonce: u64,
}

impl Transaction {
    pub fn hash(&self) -> TransactionHash {
        sha256(self)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SignedTransaction {
    pub tx: Transaction,
    #[serde(with = "serde_big_array::BigArray")]
    pub signature: Signature,
    pub public_key: PublicKeyBytes,
}

pub fn validate(state: &State, signed_tx: &SignedTransaction) -> Result<(), ValidationError> {
    let sender_address = &signed_tx.tx.from;
    let total_amount = signed_tx
        .tx
        .amount
        .checked_add(signed_tx.tx.fee)
        .ok_or(ValidationError::InsufficientBalance)?;

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

    if *sender_balance < total_amount {
        return Err(ValidationError::InsufficientBalance);
    }

    if signed_tx.tx.amount == 0 {
        return Err(ValidationError::ZeroAmount);
    }

    if signed_tx.tx.fee == 0 {
        return Err(ValidationError::ZeroFee);
    }

    // Verify that the public key matches the claimed sender address
    let derived_address = address_from_pubkey(&signed_tx.public_key);
    if derived_address != *sender_address {
        return Err(ValidationError::InvalidPublicKey);
    }

    // Verify the signature - now passes hash directly instead of transaction
    let tx_hash = signed_tx.tx.hash();
    if !verify_signature(&tx_hash, &signed_tx.signature, &signed_tx.public_key) {
        return Err(ValidationError::InvalidSignature);
    }

    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationError {
    SenderAccountNotFound,
    NonceNotFound,
    InvalidNonce,
    InsufficientBalance,
    ZeroAmount,
    ZeroFee,
    InvalidPublicKey,
    InvalidSignature,
}
