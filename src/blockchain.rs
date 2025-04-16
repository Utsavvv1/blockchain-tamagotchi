pub mod block;

// Re-export the Block struct
pub use block::Block;

// Create wrapper functions to expose the Block methods
pub fn create_genesis_block() -> Block {
    Block::create_genesis_block()
}

pub fn create_block(action: &str, previous_block: &Block) -> Block {
    Block::create_block(action, previous_block)
}