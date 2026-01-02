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
1. Conservation of value: Σ inputs − Σ outputs = 0 (ignoring fees for now)
2. Deterministic execution: Same ordered transactions ⇒ same resulting state
3. Atomicity: Invalid transaction ⇒ no partial state changes
4. Replayability: State = apply(genesis, blocks[0..n])
> If any of these fail, the chain is invalid.

> A transaction is valid iff:
1. Signature is valid
2. Sender balance ≥ amount
3. tx.nonce == account.nonce + 1
4. Amount > 0

How do we prevent double spend?? Because of nonce.
old nonce => invalid transaction (double spend detected)
