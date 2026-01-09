# Blockchain
Determistic State Machine

## How does Hashing play an important role:
1. Integrity : same i/p -> same hash
2. Tamper Evidence : any change breaks hash links
3. Content Addressing : Data refernced by what it is (and not where it is)
4. Verfiablity : Anyone can recompute and check

> Q. If two nodes process the same transactions in a different order, what breaks?
A: Consensus breaks (initialization and operation on a variable) can cause data inconsistency and security vulnerablity
i.e **DETERMINISM** fails.
- Order matters because transactions are not commutative
- Different order → different balances → different block/state hash
- Once state hashes differ, nodes cannot agree on history


## Protocol Rules:
1. Conservation of value: Σ inputs = Σ outputs + Σ fees
2. Deterministic execution: Same ordered transactions ⇒ same resulting state
3. Atomicity: Invalid transaction ⇒ no partial state changes
4. Replayability: State = apply(genesis, blocks[0..n])
> If any of these fail, the chain is invalid.

> A transaction is valid iff:
1. Signature is valid
2. Sender balance ≥ amount + fee
3. tx.nonce == account.nonce + 1
4. Amount > 0
5. Fee > 0

How do we prevent double spend?? Because of nonce.
old nonce => invalid transaction (double spend detected)

Q. Why transactions should be ordered??
A: Because consensus is about agreeing on the ordered log of state transitions, not just the final state; different transaction orders can lead to the same state but represent different histories, which breaks verifiable replay and fork resolution.

## Validate Function

Checks transaction validity **before** applying it to state. Returns `Result<(), ValidationError>`.

**Validation checks:**
1. Sender account exists in state
2. Sender nonce exists
3. Transaction nonce equals current nonce + 1 (prevents replay attacks)
4. Sender has sufficient balance for amount + fee
5. Transfer amount is greater than zero
6. Fee is greater than zero
7. Public key matches sender address
8. Signature is valid

**Important:** If validation fails, state remains unchanged.

## Apply Function

Applies a **validated** transaction to state. Never fails - assumes validation passed.

**State mutations:**
1. Deduct amount + fee from sender balance
2. Increment sender nonce
3. Add amount to receiver balance (creates account if needed)
4. Initialize receiver nonce to 0 if new account

**Note:** Fees are collected separately by the block producer.

**Contract:** Must only be called after successful validation. Panics indicate programmer error.

## Block Function

Hash previous hash, transactions, index, producer, and nonce.

A block is valid iff:
1. block.index == last.index + 1
2. block.prev_hash == last.hash()
3. block has valid Proof-of-Work (except genesis)
4. Every transaction:
    - validates against current state
    - applies cleanly in order

If any step fails → reject the block.

## Proof-of-Work (PoW)

**Difficulty:** `DIFFICULTY_LEADING_ZERO_BYTES = 2`
- Requires 2 leading zero bytes in block hash
- Equivalent to 16 leading zero bits
- Unit is **bytes**, not bits or hex characters

**Mining process:**
1. Set `block.producer` to miner's address
2. Increment `block.nonce` until hash has required leading zeros
3. Hash commits to: index, prev_hash, producer, nonce, transactions

**Critical invariant:** Producer address is part of the hash.
- Changing producer invalidates PoW
- Work is cryptographically bound to reward recipient
- Prevents PoW reuse attacks

**Genesis block:** PoW validation is skipped for block index 0.

## Network Fees

**Fee distribution:**
- Each transaction includes a mandatory fee field
- Total fees in block = Σ transaction fees
- All fees are awarded to block.producer
- Fees are added to producer's balance after block validation

**Incentive model:**
- Miners earn fees for including transactions
- Higher fees → higher priority for inclusion
- Fee market emerges naturally

## Fork & Reorgs

### Fork Choice Rules : Constant Work Addition
The canonical chain is the one with the most accumulated work.

- before: accumulated work = number of blocks
- now: addition of accumulated work (block work is constant == 1)
- later/ good implementation → sum of difficulty

### Chain Structure
```rust
pub struct Chain {
    pub blocks: HashMap<BlockHash, Block>,
    pub meta: HashMap<BlockHash, BlockMeta>,
    pub tip: BlockHash,
    pub genesis_state: State,
    pub state: State,
}
```

> **Replay > Undo** 
- Reply from genesis
- Undo is complex, bug-prone & concensus risky

### Fork Choice Algo
When a new block arrives:

1. Verify PoW
2. Verify parent exists
3. Compute height = parent.height + 1
4. Insert block into block store
5. Update chain tips
6. Select best tip (highest height)
7. If best tip ≠ current tip → **reorg**

### Reorg Algo
1. Find Common Ancestor
2. Rebuild State
3. Replace Canonical Chain

**Update** : canonical tip, canonical blocks vector, cached state <br>
**No Partial Undo** <br>
**If you cannot mathematically invert every state transition, you must replay**

## Chain
All of the validation and apply happens when we are adding a block to chain.

## Mempool

### Implement Mempool

```rust
pub struct Mempool {
    // account -> nonce -> tx
    pub by_account: HashMap<Address, BTreeMap<u64, SignedTransaction>>,

    // fee priority (fee, tx_hash)
    pub priority: BTreeMap<u64, Vec<TransactionHash>>,
}
```

### Mempool Admission Rules
1. Stateless Check (Sign, fee and amount)
2. State Relative Check (Nonce, Balance can cover amount + fee)
3. Insert to mempool

- Nonce should be increasing step by step for account

> Mempool must:
    1. Validate transactions against current state
    2. Enforce nonce ordering per account
    3. Prevent double-spend locally
    4. Order transactions by fee priority
    5. Evict transactions when a reorg happens

Q. Why mempool validation must be stricter than block validation?
A. Mempool validation must be stricter than block validation because blocks must never fail validation once mined, while mempool transactions are speculative; rejecting bad or borderline transactions early prevents miners from wasting work on blocks that would be invalid and protects the node from DoS and resource exhaustion.


## Directory Structure 

```
src/
├── main.rs              # Entry point (3 lines)
├── lib.rs               # Library root
├── core/
│   ├── mod.rs           # Module exports
│   ├── types.rs         # Type aliases & constants
│   ├── state.rs         # State struct + apply()
│   ├── transaction.rs   # Transaction, SignedTransaction, validate(), ValidationError
│   ├── block.rs         # Block, BlockMeta + hash(), work()
│   ├── consensus.rs     # Chain, ChainError + insert_block(), reorg_to()
│   ├── pow.rs           # mining(), valid_pow(), valid_hash()
│   └── mempool.rs       # Mempool, MempoolError + assemble_block()
├── node/
│   ├── mod.rs
│   ├── node.rs
│   ├── miner.rs
│   ├── gossip.rs
│   └── seen.rs
├── net/
│   ├── mod.rs
│   ├── peer.rs
│   ├── message.rs
│   └── transport.rs
├── storage/
│   ├── mod.rs           # Module exports
│   └── block_store.rs   # BlockStore + startup()
└── crypto/
    ├── mod.rs           # Module exports
    └── signature.rs     # address_from_pubkey(), verify_signature()
```

## Dependancy Flow

crypto/          (Layer 1 - Pure utilities, no imports)
  ↓
core/types       (Layer 2 - Type definitions)
  ↓
core/state       (Layer 3 - State management)
  ↓
core/transaction (Layer 3 - Uses crypto utilities)
core/block       (Layer 3)
  ↓
core/consensus   (Layer 4 - Orchestrates everything)
core/pow         (Layer 4)
core/mempool     (Layer 4 - Uses state, not chain)
  ↓
storage/         (Side layer - Uses core types, not consensus)