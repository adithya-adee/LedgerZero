use ed25519_dalek::{Signature as Ed25519Signature, Verifier, VerifyingKey};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::io::{Read, Seek, SeekFrom};
use std::{collections::HashMap, io::Write};

pub type Address = [u8; 32];
pub type Signature = [u8; 64];
pub type BlockHash = [u8; 32];
pub type TransactionHash = [u8; 32];
pub type PublicKeyBytes = [u8; 32];

pub const GENESIS_HASH: BlockHash = [0u8; 32];

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Chain {
    pub blocks: Vec<Block>,
    pub state: State,
}

impl Chain {
    pub fn new(genesis_state: State) -> Self {
        let genesis_block = Block {
            index: 0,
            prev_hash: GENESIS_HASH,
            transactions: vec![],
        };

        Self {
            blocks: vec![genesis_block],
            state: genesis_state,
        }
    }

    pub fn add_block(&mut self, block: Block) -> Result<(), ChainError> {
        let last_block = self.blocks.last().unwrap();

        if block.index != last_block.index + 1 {
            return Err(ChainError::InvalidIndex);
        }

        if block.prev_hash != last_block.hash() {
            return Err(ChainError::InvalidPreviousHash);
        }

        for tx in &block.transactions {
            validate(&self.state, tx).map_err(ChainError::TransactionValidationFailed)?;
        }

        for tx in &block.transactions {
            apply(&mut self.state, tx);
        }

        self.blocks.push(block);
        Ok(())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Block {
    pub index: u64,
    pub prev_hash: BlockHash,
    pub transactions: Vec<SignedTransaction>,
}

impl Block {
    pub fn hash(&self) -> BlockHash {
        let bytes = postcard::to_allocvec(self).expect("block serialization must be deterministic");

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

impl Transaction {
    pub fn hash(&self) -> TransactionHash {
        let bytes = postcard::to_allocvec(self).expect("tx serialization must be deterministic");

        let mut hasher = Sha256::new();
        hasher.update(bytes);
        hasher.finalize().into()
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

    // Verify that the public key matches the claimed sender address
    let derived_address = address_from_pubkey(&signed_tx.public_key);
    if derived_address != *sender_address {
        return Err(ValidationError::InvalidPublicKey);
    }

    // Verify the signature
    if !verify_signature(&signed_tx.tx, &signed_tx.signature, &signed_tx.public_key) {
        return Err(ValidationError::InvalidSignature);
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationError {
    SenderAccountNotFound,
    NonceNotFound,
    InvalidNonce,
    InsufficientBalance,
    ZeroAmount,
    InvalidPublicKey,
    InvalidSignature,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChainError {
    InvalidIndex,
    InvalidPreviousHash,
    TransactionValidationFailed(ValidationError),
}

fn main() {
    println!("LedgerZero");
}

fn address_from_pubkey(pubkey: &PublicKeyBytes) -> Address {
    let mut hasher = Sha256::new();
    hasher.update(pubkey);
    hasher.finalize().into()
}

fn verify_signature(tx: &Transaction, sig_bytes: &Signature, pubkey: &PublicKeyBytes) -> bool {
    let tx_hash = tx.hash();

    // Parse the public key bytes into a VerifyingKey
    let verifying_key = match VerifyingKey::from_bytes(pubkey) {
        Ok(vk) => vk,
        Err(_) => return false,
    };

    // Convert our signature bytes to ed25519_dalek::Signature
    let signature = Ed25519Signature::from_bytes(sig_bytes);

    // Verify the signature against the transaction hash
    verifying_key.verify(&tx_hash, &signature).is_ok()
}

// PERSISTENCE

pub struct BlockStore {
    file: std::fs::File,
}

impl BlockStore {
    /// Creates or opens the block store in the ledger_data directory
    pub fn new() -> std::io::Result<Self> {
        // Create the data directory if it doesn't exist
        std::fs::create_dir_all("ledger_data")?;
        
        // Open or create the blocks file with read/write permissions
        let file = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open("ledger_data/blocks.dat")?;
        
        Ok(BlockStore { file })
    }

    pub fn append_block(&mut self, block: &Block) -> std::io::Result<()> {
        let block_bytes =
            postcard::to_allocvec(block).expect("block serialization must be deterministic");

        let len = block_bytes.len() as u32;

        // Seek to end before writing
        self.file.seek(SeekFrom::End(0))?;
        self.file.write_all(&len.to_le_bytes())?;
        self.file.write_all(&block_bytes)?;
        self.file.sync_all()?;

        Ok(())
    }

    pub fn load_blocks(&mut self) -> std::io::Result<Vec<Block>> {
        let mut blocks = Vec::new();
        self.file.seek(SeekFrom::Start(0))?;

        loop {
            let mut len_buf = [0u8; 4];
            if self.file.read_exact(&mut len_buf).is_err() {
                break; // EOF
            }

            let len = u32::from_le_bytes(len_buf) as usize;
            let mut buf = vec![0u8; len];
            self.file.read_exact(&mut buf)?;

            let block: Block = postcard::from_bytes(&buf).expect("stored block must deserialize");

            blocks.push(block);
        }

        Ok(blocks)
    }
}

pub fn startup(mut store: BlockStore, genesis_state: State) -> std::io::Result<Chain> {
    let blocks = store.load_blocks()?;
    
    // If no blocks exist, return a new chain with genesis
    if blocks.is_empty() {
        return Ok(Chain::new(genesis_state));
    }
    
    let mut chain = Chain::new(genesis_state);

    // Skip the genesis block (index 0) and replay all other blocks
    for block in blocks.into_iter().skip(1) {
        chain.add_block(block).expect("invalid chain on disk");
    }
    
    Ok(chain)
}
