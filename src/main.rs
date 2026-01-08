use ed25519_dalek::{Signature as Ed25519Signature, Verifier, VerifyingKey};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::io::{Read, Seek, SeekFrom};
use std::{collections::HashMap, io::Write};

pub type Address = [u8; 32];
pub type Signature = [u8; 64];
pub type BlockHash = [u8; 32];
pub type TransactionHash = [u8; 32];
pub type PublicKeyBytes = [u8; 32];

pub const GENESIS_HASH: BlockHash = [0u8; 32];
pub const ZERO_ADDRESS: Address = [0u8; 32];
pub const DIFFICULTY_PREFIX_ZERO_BYTES: usize = 2;

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

        account_pool.insert(tx_nonce, tx);

        Ok(())
    }
}

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

    pub fn insert_block(&mut self, block: Block) -> Result<(), ChainError> {
        if block.index != 0 && !valid_pow(&block) {
            return Err(ChainError::InvalidPoW);
        }

        let parent_hash = block.prev_hash;

        if block.index != 0 && !self.blocks.contains_key(&parent_hash) {
            return Err(ChainError::UnknownParent);
        }

        let parent_height = if block.index == 0 {
            0
        } else {
            self.meta[&parent_hash].height
        };

        let height = parent_height + 1;
        let hash = block.hash();
        let total_work = if block.index == 0 {
            0 // Genesis block has no accumulated work
        } else {
            self.meta[&parent_hash].total_work + block.work()
        };

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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Block {
    pub index: u64,
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

pub fn mining(block: &mut Block) {
    loop {
        let hash = block.hash();

        if valid_hash(hash) {
            break;
        }

        block.nonce += 1;
    }
}

fn valid_hash(hash: [u8; 32]) -> bool {
    for i in 0..DIFFICULTY_PREFIX_ZERO_BYTES {
        if hash[i] != 0 {
            return false;
        }
    }
    true
}

pub fn valid_pow(block: &Block) -> bool {
    valid_hash(block.hash())
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
    pub fee: u64,
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

pub fn assemble_block(chain: &Chain, mempool: &Mempool, producer: Address) -> Block {
    let mut temp_state = chain.state.clone();
    let mut transactions: Vec<SignedTransaction> = Vec::new();

    for (_addrs, map) in &mempool.by_account {
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
        index: 2,
        prev_hash: chain.tip,
        producer,
        nonce: 0,
        transactions,
    }
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MempoolError {
    InvalidTransaction(ValidationError),
    NonceTooLow,
    NonceGap,
    InsufficientBalance,
    DuplicateTransaction,
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChainError {
    InvalidIndex,
    InvalidPreviousHash,
    InvalidPoW,
    UnknownParent,
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
    let genesis_block = Block {
        index: 0,
        prev_hash: GENESIS_HASH,
        producer: ZERO_ADDRESS,
        nonce: 0,
        transactions: vec![],
    };

    // If no blocks exist, return a new chain with genesis
    if blocks.is_empty() {
        return Ok(Chain::new(genesis_state, genesis_block));
    }

    let mut chain = Chain::new(genesis_state, genesis_block);

    // Skip the genesis block (index 0) and replay all other blocks
    for block in blocks.into_iter().skip(1) {
        chain.insert_block(block).expect("invalid chain on disk");
    }

    Ok(chain)
}
