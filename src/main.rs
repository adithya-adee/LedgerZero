use std::sync::{Arc, RwLock};
use std::thread;
use std::time::Duration;

use LedgerZero::core::block::Block;
use LedgerZero::core::consensus::Chain;
use LedgerZero::core::state::State;
use LedgerZero::core::types::{Address, GENESIS_HASH, ZERO_ADDRESS};
use LedgerZero::net::transport::Server;
use LedgerZero::node::miner::start_mining;
use LedgerZero::node::node::Node;

fn main() {
    println!("🚀 Starting LedgerZero Node...\n");

    // Create genesis state with some initial balances for testing
    let mut genesis_state = State::new();
    let miner_address: Address = [1u8; 32]; // Miner's address

    // Give miner some initial balance for fees
    genesis_state.balances.insert(miner_address, 1000000);
    genesis_state.nonces.insert(miner_address, 0);

    // Create genesis block
    let genesis_block = Block {
        prev_hash: GENESIS_HASH,
        producer: ZERO_ADDRESS,
        nonce: 0,
        transactions: vec![],
    };

    // Initialize chain
    let chain = Chain::new(genesis_state, genesis_block);
    let node = Node::new(chain);

    println!("✓ Genesis block created");
    println!("✓ Miner address: {:?}...\n", &miner_address[..8]);

    // Create server
    let mut server = Server::new(node);

    // Start TCP listener
    let listen_addr = "127.0.0.1:8001";
    server
        .start_listener(listen_addr)
        .expect("Failed to start listener");
    println!("✓ Listening on {}\n", listen_addr);

    // Wrap server in Arc<RwLock> for sharing with miner
    let server = Arc::new(RwLock::new(server));

    // Start mining in background
    let mining_interval_ms = 5000; // Mine every 5 seconds
    let _miner_handle = start_mining(Arc::clone(&server), miner_address, mining_interval_ms);

    println!("✓ Mining started (interval: {}ms)\n", mining_interval_ms);
    println!("Node is running. Press Ctrl+C to stop.\n");
    println!("Commands:");
    println!("  - Add peers by running another instance on port 8002");
    println!("  - The node will automatically mine and broadcast blocks\n");

    // Print stats periodically
    loop {
        thread::sleep(Duration::from_secs(10));

        let server_lock = server.read().unwrap();
        let node = server_lock.node();
        let node_lock = node.read().unwrap();

        println!("📊 Stats:");
        println!("   Chain height: TBD"); // Need to add height tracking
        println!("   Mempool txs: {}", node_lock.mempool.by_account.len());
        println!("   Connected peers: {}", server_lock.peer_count());
        println!();
    }
}

#[cfg(test)]
mod tests {
    use LedgerZero::core::block::Block;
    use LedgerZero::core::consensus::Chain;
    use LedgerZero::core::mempool::assemble_block;
    use LedgerZero::core::pow;
    use LedgerZero::core::state::State;
    use LedgerZero::core::transaction::{SignedTransaction, Transaction};
    use LedgerZero::core::types::{Address, GENESIS_HASH, ZERO_ADDRESS};
    use LedgerZero::crypto::signature::address_from_pubkey;
    use LedgerZero::node::node::Node;

    use ed25519_dalek::{Signer, SigningKey};

    /// Helper to create a keypair and derive address
    fn create_user() -> (SigningKey, Address) {
        let mut seed = [0u8; 32];
        rand::fill(&mut seed);
        let signing_key = SigningKey::from_bytes(&seed);
        let public_key = signing_key.verifying_key().to_bytes();
        let address = address_from_pubkey(&public_key);
        (signing_key, address)
    }
    /// Helper to create and sign a transaction
    fn create_signed_tx(
        signing_key: &SigningKey,
        from: Address,
        to: Address,
        amount: u64,
        fee: u64,
        nonce: u64,
    ) -> SignedTransaction {
        let tx = Transaction {
            from,
            to,
            amount,
            fee,
            nonce,
        };
        let tx_hash = tx.hash();
        let signature = signing_key.sign(&tx_hash);
        
        SignedTransaction {
            tx,
            signature: signature.to_bytes(),
            public_key: signing_key.verifying_key().to_bytes(),
        }
    }

    /// Create genesis block
    fn create_genesis_block() -> Block {
        Block {
            prev_hash: GENESIS_HASH,
            producer: ZERO_ADDRESS,
            nonce: 0,
            transactions: vec![],
        }
    }

    #[test]
    fn test_mempool_and_chain_flow() {
        // Setup: Create users
        let (user1_key, user1_addr) = create_user();
        let (_user2_key, user2_addr) = create_user();
        let miner_addr: Address = [99u8; 32];

        // Create genesis state with user1 having balance
        let mut genesis_state = State::new();
        genesis_state.balances.insert(user1_addr, 1000);
        genesis_state.nonces.insert(user1_addr, 0);
        genesis_state.balances.insert(miner_addr, 0);
        genesis_state.nonces.insert(miner_addr, 0);

        // Create genesis block and chain
        let genesis_block = create_genesis_block();
        let chain = Chain::new(genesis_state.clone(), genesis_block);
        let mut node = Node::new(chain);

        println!("✓ Genesis setup complete");
        println!("  User1 balance: {}", node.chain.state.balances.get(&user1_addr).unwrap());

        // Create and submit transaction: user1 -> user2
        let signed_tx = create_signed_tx(&user1_key, user1_addr, user2_addr, 100, 1, 1);
        
        // Add to mempool
        let result = node.on_transaction(signed_tx.clone());
        assert!(result.is_ok(), "Transaction should be added to mempool");
        
        // Verify transaction is in mempool
        let mempool_txs = node.mempool.by_account.get(&user1_addr);
        assert!(mempool_txs.is_some(), "User1 should have pending txs in mempool");
        assert_eq!(mempool_txs.unwrap().len(), 1, "Should have exactly 1 tx");

        println!("✓ Transaction added to mempool");

        // Assemble block from mempool
        let mut block = assemble_block(
            &node.chain.state,
            node.chain.tip,
            &node.mempool,
            miner_addr,
        );
        
        assert_eq!(block.transactions.len(), 1, "Block should contain 1 tx");
        println!("✓ Block assembled with {} transactions", block.transactions.len());

        // Mine the block (find valid PoW nonce)
        pow::mining(&mut block).expect("Mining should succeed");
        println!("✓ Block mined with nonce: {}", block.nonce);

        // Add block to chain
        let result = node.on_block(block.clone());
        assert!(result.is_ok(), "Block should be added to chain");

        // Verify state was updated
        let user1_new_balance = *node.chain.state.balances.get(&user1_addr).unwrap();
        let user2_balance = *node.chain.state.balances.get(&user2_addr).unwrap_or(&0);
        let miner_balance = *node.chain.state.balances.get(&miner_addr).unwrap();

        println!("✓ Chain updated");
        println!("  User1 balance: {} (was 1000)", user1_new_balance);
        println!("  User2 balance: {}", user2_balance);
        println!("  Miner balance: {} (fees)", miner_balance);

        assert_eq!(user1_new_balance, 1000 - 100 - 1, "User1 should have 899");
        assert_eq!(user2_balance, 100, "User2 should have 100");
        assert_eq!(miner_balance, 1, "Miner should have 1 (fee)");

        // Verify mempool was cleared
        assert!(
            node.mempool.by_account.get(&user1_addr).is_none() ||
            node.mempool.by_account.get(&user1_addr).unwrap().is_empty(),
            "Mempool should be empty after block"
        );

        println!("\n✓ All assertions passed!");
    }

    #[test]
    fn test_two_nodes_consensus() {
        // Setup: Create users
        let (user1_key, user1_addr) = create_user();
        let (_user2_key, user2_addr) = create_user();
        let miner_addr: Address = [99u8; 32];

        // Create identical genesis state for both nodes
        let mut genesis_state = State::new();
        genesis_state.balances.insert(user1_addr, 1000);
        genesis_state.nonces.insert(user1_addr, 0);
        genesis_state.balances.insert(miner_addr, 0);
        genesis_state.nonces.insert(miner_addr, 0);

        let genesis_block = create_genesis_block();

        // Create two nodes with same genesis
        let chain1 = Chain::new(genesis_state.clone(), genesis_block.clone());
        let chain2 = Chain::new(genesis_state.clone(), genesis_block.clone());
        
        let mut node1 = Node::new(chain1);
        let mut node2 = Node::new(chain2);

        // Verify both nodes start with same tip
        assert_eq!(node1.chain.tip, node2.chain.tip, "Both nodes should have same genesis tip");
        println!("✓ Both nodes have same genesis");

        // Create transaction on node1
        let signed_tx = create_signed_tx(&user1_key, user1_addr, user2_addr, 50, 1, 1);

        // Add to node1's mempool
        node1.on_transaction(signed_tx.clone()).unwrap();
        
        // Also add to node2's mempool (simulating gossip)
        node2.on_transaction(signed_tx).unwrap();

        println!("✓ Transaction in both mempools");

        // Node1 mines a block
        let mut block = assemble_block(
            &node1.chain.state,
            node1.chain.tip,
            &node1.mempool,
            miner_addr,
        );
        pow::mining(&mut block).unwrap();

        // Node1 adds block to its chain
        node1.on_block(block.clone()).unwrap();
        println!("✓ Node1 mined and added block");

        // Node2 receives and adds the block (simulating gossip)
        node2.on_block(block).unwrap();
        println!("✓ Node2 received and added block");

        // Verify both nodes have same chain tip
        assert_eq!(node1.chain.tip, node2.chain.tip, "Both nodes should have same tip after sync");
        
        // Verify both nodes have same state
        let node1_user2_bal = *node1.chain.state.balances.get(&user2_addr).unwrap_or(&0);
        let node2_user2_bal = *node2.chain.state.balances.get(&user2_addr).unwrap_or(&0);
        assert_eq!(node1_user2_bal, node2_user2_bal, "Both nodes should have same user2 balance");
        assert_eq!(node1_user2_bal, 50, "User2 should have 50");

        println!("\n✓ Both nodes in consensus!");
        println!("  Common chain tip: {:?}...", &node1.chain.tip[..8]);
        println!("  User2 balance on both: {}", node1_user2_bal);
    }
}
