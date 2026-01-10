use std::sync::{Arc, RwLock};
use std::thread;
use std::time::Duration;

use crate::core::mempool::assemble_block;
use crate::core::pow;
use crate::core::types::Address;
use crate::net::transport::Server;

/// Start mining in a background thread
/// Continuously assembles blocks from mempool, mines them, and broadcasts to peers
pub fn start_mining(
    server: Arc<RwLock<Server>>,
    producer_address: Address,
    mine_interval_ms: u64,
) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        loop {
            // Small delay between mining attempts
            thread::sleep(Duration::from_millis(mine_interval_ms));

            // Assemble block from current state
            let mut block = {
                let server_lock = server.read().unwrap();
                let node = server_lock.node();
                let node_lock = node.read().unwrap();

                let chain_tip = node_lock.chain.tip;
                let chain_state = &node_lock.chain.state;
                let mempool = &node_lock.mempool;

                assemble_block(chain_state, chain_tip, mempool, producer_address)
            };

            // Mine the block (find valid nonce)
            if pow::mining(&mut block).is_ok() {
                println!(
                    "✓ Mined block with {} transactions, nonce: {}",
                    block.transactions.len(),
                    block.nonce
                );

                // Add block to our own chain
                let node = {
                    let server_lock = server.read().unwrap();
                    server_lock.node()
                };

                if let Err(e) = node.write().unwrap().on_block(block.clone()) {
                    eprintln!("Failed to add mined block to chain: {:?}", e);
                    continue;
                }

                // Broadcast to peers
                let server_lock = server.read().unwrap();
                if let Err(e) = server_lock.broadcast_block(&block) {
                    eprintln!("Failed to broadcast block: {:?}", e);
                }
            }
        }
    })
}