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


## Chain
All of the validation and apply happens when we are adding a block to chain.