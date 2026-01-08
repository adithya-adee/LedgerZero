use crate::core::{block::Block, types::DIFFICULTY_PREFIX_ZERO_BYTES};

pub fn mining(block: &mut Block) -> Result<(), ()> {
    loop {
        let hash = block.hash();

        if valid_hash(hash) {
            return Ok(());
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
