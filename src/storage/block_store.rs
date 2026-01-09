use crate::core::block::Block;
use std::io::{Read, Seek, SeekFrom, Write};

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
