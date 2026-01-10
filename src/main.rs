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
