pub type Address = [u8; 32];
pub type Signature = [u8; 64];
pub type BlockHash = [u8; 32];
pub type TransactionHash = [u8; 32];
pub type PublicKeyBytes = [u8; 32];

pub const GENESIS_HASH: BlockHash = [0u8; 32];
pub const ZERO_ADDRESS: Address = [0u8; 32];
pub const DIFFICULTY_PREFIX_ZERO_BYTES: usize = 2;
